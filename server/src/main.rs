#![feature(await_macro, async_await, futures_api)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tokio;

mod game_server;
mod map_server;
mod game;

use self::game_server::map_manager;
use self::game_server::GameServer;
use self::map_server::MapServer;

fn main() {
    println!("Hello, world!");
    bootstrap_map_server();
    bootstrap_game_servers()
}

/// Starts up the game server
fn bootstrap_game_servers() {
    GameServer::start(
        1234,
        map_manager::FileSystemMapManager::new(
            "Q:\\Projects\\Development\\2019\\inter-planet-game\\maps".to_string(),
        ),
    );
}

fn bootstrap_map_server() {
    MapServer::start(
        5665,
        map_manager::FileSystemMapManager::new(
            "Q:\\Projects\\Development\\2019\\inter-planet-game\\maps".to_string(),
        ),
    )
}