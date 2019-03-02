use ws::{listen, Handler, Sender, Handshake, Result, Message};
use crate::game::{ Game };
use std::collections::HashMap;
use tokio;
pub mod map_manager;

pub struct Server {
    out: Sender,
}

impl Handler for Server {
    fn on_open (&mut self, _:Handshake) -> Result <()> {
        Ok(())
    }

    fn on_message (&mut self, msg: Message) -> Result<()> {
        println!("Recivied Message: {}", msg.as_text().unwrap());
        tokio::run_async(async move {
            
        });
        // TODO match messsage type
        // Planned types
        // Set Name
        // Enter Game

        // Game Move
        // Exit Game
        Ok(())
    }
}

pub struct GameServer {
    games: HashMap<String,Game>,
    map_manager: Box<map_manager::MapManager>
}

impl GameServer {
    /// Initializes a game server
    /// 
    /// # Examples
    /// 
    /// ```
    /// let port: u16 = 1234;
    /// // Websocket now can be reached from localhost:1234
    /// GameServer::new(port);
    /// 
    pub fn new (port: u16) -> GameServer {
        let instance = GameServer {
            games: HashMap::new(),
            map_manager: Box::new(map_manager::FileSystemMapManager::new(String::from("./maps")))
        };

        listen(format!("127.0.0.1:{}", port), |out| {
            Server { out: out }
        }).unwrap();
        instance
    }
}
