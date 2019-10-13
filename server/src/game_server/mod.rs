use tokio::prelude::*;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use ipg_core::game::Game;
use std::collections::HashMap;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::future::Future;
use std::iter;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::WebSocketStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
pub mod map_manager;
pub mod connection;
use self::connection::GameConnection;

use futures::sync::mpsc::SendError;
use ipg_core::protocol::messages::GameMetadata;
use ipg_core::game::GameExecutor;
use std::convert::Into;

pub struct GameServer {
    port: u16,
    games: RwLock<HashMap<String, Arc<Mutex<GameExecutor>>>>,
    map_manager: Mutex<Box<map_manager::MapManager + Send>>,
}

trait GameList {
    fn add_game(&mut self, game: Game) -> String;
}

impl GameList for HashMap<String, Arc<Mutex<GameExecutor>>>
{
    fn add_game(&mut self, game: Game) -> String {
        let game_id: String = iter::repeat(())
            .map(|()| thread_rng().sample(Alphanumeric))
            .take(7)
            .collect();
        self.insert(game_id.clone(), Arc::new(Mutex::new(GameExecutor::from_game(game))));
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
            games: RwLock::new(HashMap::new()),
            map_manager: Mutex::new(Box::new(maps)),
        };

        let listener = TcpListener::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
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

    /// Handles an incoming websocket stream
    /// It will mutate game state based in incoming messages,
    /// and broadcast the messages the client requires.
    ///
    async fn handle_stream<'a>(instance: Arc<GameServer>, ws_stream: WebSocketStream<TcpStream>) {
        let (mut sink, mut stream) = ws_stream.split();
        tokio::spawn_async(async move {
            let mut connection = GameConnection::new(instance.clone(),sink);
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
        })
    }
}
