use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use ipg_core::game::Game;
use ipg_core::protocol::messages::{MessageType,GameMetadata};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::future::Future;
use std::iter;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::WebSocketStream;
pub mod connection;
pub mod map_manager;
use self::connection::GameConnection;

use futures::sync::mpsc::SendError;
use ipg_core::game::GameExecutor;
use std::convert::Into;

pub struct GameServer {
    port: u16,
    games: RwLock<HashMap<String, Arc<Mutex<GameExecutor>>>>,
    // In theory, the sinks will end up all being the same type, meaning static dispatch is not out of the quesiton.
    connections: Mutex<Vec<Box<Arc<Mutex<dyn Sink<SinkItem=Message,SinkError=Error> + Send + Sync>>>>>,
    map_manager: Mutex<Box<map_manager::MapManager + Send>>,
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
            Arc::new(Mutex::new(GameExecutor::from_game(game,game_id.clone()))),
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
        let mut instance = GameServer {
            port: port,
            connections: Mutex::new(Vec::new()),
            games: RwLock::new(HashMap::new()),
            map_manager: Mutex::new(Box::new(maps)),
        };

        let listener = TcpListener::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port,
        ))
        .unwrap();
        tokio::run_async(async move {
            let mut incoming = listener.incoming();
            let shareable_instance = Arc::new(instance);
            while let Some(stream) = await!(incoming.next()) {
                let stream = stream.unwrap();
                let ws_stream = await!(accept_async(stream));
                await!(GameServer::handle_stream(
                    shareable_instance.clone(),
                    ws_stream.unwrap()
                ));
            }
        });
    }

    pub fn add_game(&self, game: Game) -> String {
        let mut games = self.games.write().unwrap();
        let map_id = game.map.name.clone();
        let game_id = games.add_game(game);
        let message = Message::from(
            serde_json::to_string(&MessageType::NewGame(GameMetadata {
                game_id: game_id.clone(),
                map_id,
            })).unwrap(),
        );
        self.broadcast(message);
        game_id
    }

    fn broadcast(&self, message: Message) {
        for connection in self.connections.lock().unwrap().iter() {
            connection.lock().unwrap().start_send(message.clone());
        };
    }

    pub fn remove_game(&self, game_id: &String) {
        let mut games = self.games.write().unwrap();
        games.remove(game_id);
        let message = Message::from(
            serde_json::to_string(&MessageType::RemoveGame(game_id.clone())).unwrap(),
        );
        self.broadcast(message);
    }

    /// Handles an incoming websocket stream
    /// It will mutate game state based in incoming messages,
    /// and broadcast the messages the client requires.
    ///
    async fn handle_stream<'a>(instance: Arc<GameServer>, ws_stream: WebSocketStream<TcpStream>) {
        let (mut sink, mut stream) = ws_stream.split();
        let sink_mtx = Arc::new(Mutex::new(sink));
        // New scope to make sure the lock gets dropped immediately
        { 
            instance.connections.lock().unwrap().push(Box::new(sink_mtx.clone()));
        };
        println!("Connection opened.");
        tokio::spawn_async(async move {
            let mut connection = GameConnection::new(instance.clone(), sink_mtx.clone());
            await!(connection.handle_new_client());
            while let Some(Ok(message)) = await!(stream.next()) {
                let message = message;
                await!(connection.handle_message(&message));
                if let Ok(text) = message.into_text() {
                    // sink.start_send(Message::from(format!(
                    //     "You sent a message containing {}.",
                    //     text
                    // )))
                    // .unwrap();
                };
            }
            connection.handle_client_exit();
            println!("Connection closed.");
        });
    }
}
