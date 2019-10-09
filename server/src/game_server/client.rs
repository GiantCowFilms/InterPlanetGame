use tokio_tungstenite::tungstenite::{Error, Message};
use futures::sink::Sink;
use futures::stream::Stream;

pub struct Client<'a> {
    pub sink: &'a mut (Sink<SinkItem = Message, SinkError = Error> + Send),
    pub stream: &'a mut (Stream<Item = Message, Error = Error> + Send)
}