use crate::game_server::messages::MessageType;
use crate::GameServer;
use futures::sink::Sink;
use futures::sync::mpsc::SendError;
use std::future::Future;
use tokio_tungstenite::tungstenite::{Error, Message};

pub trait Captures<'a> {}

impl<'a, T> Captures<'a> for T {}

impl GameServer {
    pub fn handle_message<'a: 'c, 'b: 'c, 'c>(
        &mut self,
        message: &'a Message,
        sink: &'b mut ((Sink<SinkItem = Message, SinkError = Error>) + Send),
    ) -> impl Future<Output = ()> + Captures<'a> + Captures<'b> + 'c {
        async move {
            let result = if let Ok(message_body) = message.to_text() {
                if let Ok(message_data) = serde_json::from_str::<MessageType>(message_body) {
                    match message_data {
                        MessageType::CreateGame(game_settings) => Ok(()),
                        MessageType::ExitGame => {
                            let _ = sink.start_send(Message::from("ExitGame"));
                            Ok(())
                        }
                        _ => Err("The provided message type was not found."),
                    }
                } else {
                    Err("Could not parse the provided message.")
                }
            } else {
                Err("The recieved message could not be parsed as a string.")
            };
            match result {
                Ok(_) => (),
                Err(e) => {
                    let _ = sink.start_send(Message::from(e));
                }
            };
        }
    }
}
