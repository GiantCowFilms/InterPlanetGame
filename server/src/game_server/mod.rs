use std::sync::Arc;
use crate::game::Game;
use std::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::WebSocketStream;
use std::sync::{ Mutex, RwLock };
pub mod map_manager;
pub mod message_handler;
pub mod messages;
use crate::game_server::messages::GameMetadata;
use futures::sync::mpsc::SendError;

pub struct GameServer {
    port: u16,
    games: RwLock<HashMap<String, Game>>,
    map_manager: Mutex<Box<map_manager::MapManager + Send>>,
}

trait GameList {
    fn add_game(&mut self, game: Game) -> String;
}

impl GameList for HashMap<String,Game>  {
    fn add_game(&mut self, game: Game) -> String {
        let game_id: String = iter::repeat(())
            .map(|()| thread_rng().sample(Alphanumeric))
            .take(7)
            .collect();
        self.insert(game_id.clone(),game);
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
        tokio::run_async(
            async move {
                let mut incoming = listener.incoming();
                let shareable_instance = Arc::new(instance);
                while let Some(stream) = await!(incoming.next()) {
                    let stream = stream.unwrap();
                    let ws_stream = await!(accept_async(stream));
                    await!(GameServer::handle_stream(shareable_instance.clone(),ws_stream.unwrap()));
                }
            },
        );
    }

    async fn handle_stream<'a>(instance: Arc<GameServer>, ws_stream: WebSocketStream<TcpStream>) {
        let (mut sink, mut stream) = ws_stream.split();
        let thread = tokio::spawn_async(
            async move {
                let _ = sink.start_send(Message::from("Hello World!"));
                await!(GameServer::handle_new_client(instance.clone(),&mut sink));
                while let Some(message) = await!(stream.next()) {
                    let message = message.unwrap();
                    await!(GameServer::handle_message(instance.clone(),&message, &mut sink));
                    if let Ok(text) = message.into_text() {
                        sink.start_send(Message::from(format!(
                            "You sent a message containing {}.",
                            text
                        )))
                        .unwrap();
                    };
                };
            },
        );
        thread
    }

    async fn handle_new_client(
        instance: Arc<GameServer>,
        sink: & mut ((Sink<SinkItem = Message, SinkError = Error>) + Send),
    ) {
        use crate::game_server::messages::{ GameList, MessageType };
        match (*instance).games.read() {
            Ok(mut games) => {
                let games_metadata = games.iter().map(|(key,val)| {
                    GameMetadata {
                        game_id: key.clone()
                    }
                }).collect();
                let seralized = serde_json::to_string(&MessageType::GameList(GameList {
                    games: games_metadata
                }));
                let _ = sink.start_send(Message::from(seralized.unwrap()));
                Ok(())
            }
            _=> Err("Game state corrupted by poisned mutex.".to_string())
        };
        let _ = sink.start_send(Message::from("Hello World!"));
    }
}
