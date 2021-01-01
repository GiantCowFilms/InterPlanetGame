use futures::{SinkExt, StreamExt};
use ipg_core::game::Game;
use ipg_core::protocol::messages::{GameMetadata, MessageType};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::iter;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::{mpsc, Mutex, RwLock},
};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
pub mod connection;
pub mod map_manager;
use self::connection::GameConnection;

use ipg_core::game::GameExecutor;

pub struct GameServer {
    #[allow(unused)]
    port: u16,
    games: RwLock<HashMap<String, Arc<Mutex<GameExecutor>>>>,
    // In theory, the sinks will end up all being the same type, meaning static dispatch is not out of the quesiton.
    connections: Mutex<Vec<mpsc::Sender<Message>>>,
    map_manager: Mutex<Box<dyn map_manager::MapManager + Send>>,
}

trait GameList {
    fn add_game(&mut self, game: Game) -> String;
}

impl GameList for HashMap<String, Arc<Mutex<GameExecutor>>> {
    fn add_game(&mut self, game: Game) -> String {
        let game_id: String = iter::repeat(())
            .map(|()| thread_rng().sample(Alphanumeric))
            .take(7)
            .collect();
        self.insert(
            game_id.clone(),
            Arc::new(Mutex::new(GameExecutor::from_game(game, game_id.clone()))),
        );
        game_id
    }
}

impl GameServer {
    /// Starts a game server
    ///
    /// # Examples
    ///
    /// ```
    /// let port: u16 = 1234;
    /// // Websocket now can be reached from localhost:1234
    /// GameServer::start(port);
    ///
    pub fn start(port: u16, maps: impl map_manager::MapManager + Send + 'static) {
        let instance = GameServer {
            port: port,
            connections: Mutex::new(Vec::new()),
            games: RwLock::new(HashMap::new()),
            map_manager: Mutex::new(Box::new(maps)),
        };
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let mut listener = TcpListener::bind(&SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                port,
            ))
            .await
            .unwrap();
            let shareable_instance = Arc::new(instance);
            let mut incoming = listener.incoming();
            while let Some(stream) = incoming.next().await {
                let stream = stream.unwrap();
                let ws_stream = accept_async(stream).await;
                GameServer::handle_stream(shareable_instance.clone(), ws_stream.unwrap()).await;
            }
        });
    }

    pub async fn add_game(&self, game: Game) -> String {
        let mut games = self.games.write().await;
        let map_id = game.map.name.clone();
        let config = game.config.clone();
        let game_id = games.add_game(game);
        let message = Message::from(
            serde_json::to_string(&MessageType::NewGame(GameMetadata {
                game_id: game_id.clone(),
                config,
                map_id,
            }))
            .unwrap(),
        );
        self.broadcast(message).await;
        game_id
    }

    async fn broadcast(&self, message: Message) {
        let mut connections = self.connections.lock().await;
        for connection in connections.iter_mut() {
            // TODO we should record when message sending fails (or even better
            // retry). Failing silently is not good, since it may make it
            // difficult to debug any issues that stem from message transport
            // failure.
            let _ = connection.send(message.clone()).await;
            // Per optimization possibilty - currently we wait for every message to bet sent fully before sending this next one.
            // This would be much slower then concurrently sending all the messages
        }
    }

    pub async fn remove_game(&self, game_id: &String) {
        let mut games = self.games.write().await;
        games.remove(game_id);
        let message = Message::from(
            serde_json::to_string(&MessageType::RemoveGame(game_id.clone())).unwrap(),
        );
        self.broadcast(message).await;
    }

    /// Handles an incoming websocket stream
    /// It will mutate game state based in incoming messages,
    /// and broadcast the messages the client requires.
    ///
    async fn handle_stream<'a>(instance: Arc<GameServer>, ws_stream: WebSocketStream<TcpStream>) {
        let (mut sink, mut stream) = ws_stream.split();
        // Create mpsc for sending to client so we can share the sender around
        let (tx, mut rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(message) = rx.next().await {
                let _ = sink.send(message).await;
            }
        });
        // New scope to make sure the lock gets dropped immediately
        {
            instance.connections.lock().await.push(tx.clone());
        };
        println!("Connection opened.");
        tokio::spawn(async move {
            let mut connection = GameConnection::new(instance.clone(), tx);
            connection.handle_new_client().await;
            while let Some(Ok(message)) = stream.next().await {
                let message = message;
                connection.handle_message(&message).await;
            }
            connection.handle_client_exit().await;
            println!("Connection closed.");
        });
    }
}
