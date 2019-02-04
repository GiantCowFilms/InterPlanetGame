#[macro_use]
extern crate serde_derive;

mod game_server;
mod game;
use self::game_server::GameServer;

fn main() {
    println!("Hello, world!");
    bootstrap_game_servers();
}

/// Starts up the game server
fn bootstrap_game_servers() {
    GameServer::new(1234);
}