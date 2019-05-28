#![feature(await_macro, async_await, futures_api)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tokio;

mod game;
mod game_server;
use self::game_server::GameServer;

fn main() {
    println!("Hello, world!");
    bootstrap_game_servers();
}

/// Starts up the game server
fn bootstrap_game_servers() {
    GameServer::start(1234);
}
