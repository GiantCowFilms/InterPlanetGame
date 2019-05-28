use crate::game::Game;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::WebSocketStream;
pub mod map_manager;
pub mod message_handler;
pub mod messages;

pub struct GameServer {
    port: u16,
    games: HashMap<String, Game>,
    map_manager: Box<map_manager::MapManager>,
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
    pub fn start(port: u16) -> GameServer {
        let instance = GameServer {
            port: port,
            games: HashMap::new(),
            map_manager: Box::new(map_manager::FileSystemMapManager::new(String::from(
                "./maps",
            ))),
        };

        let listener = TcpListener::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        ))
        .unwrap();
        tokio::run_async(
            async {
                let mut incoming = listener.incoming();

                while let Some(stream) = await!(incoming.next()) {
                    let stream = stream.unwrap();
                    let ws_stream = await!(accept_async(stream));
                    await!(instance.handle_stream(ws_stream.unwrap()));
                }
            },
        );
        instance
    }

    async fn handle_stream(&mut self, ws_stream: WebSocketStream<TcpStream>) {
        let (mut sink, mut stream) = ws_stream.split();
        let thread = tokio::spawn_async(
            async move {
                let _ = sink.start_send(Message::from("Hello World!"));
                while let Some(message) = await!(stream.next()) {
                    let message = message.unwrap();
                    await!(self.handle_message(&message, &mut sink));
                    if let Ok(text) = message.into_text() {
                        sink.start_send(Message::from(format!(
                            "You sent a message containing {}.",
                            text
                        )))
                        .unwrap();
                    };
                }
            },
        );
        thread
    }
}
