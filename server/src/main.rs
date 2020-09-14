#![feature(async_closure)]
extern crate serde_derive;
extern crate tokio;

mod game_server;

use self::game_server::map_manager;
use self::game_server::GameServer;

fn main() {
    println!("Hello, world!");
    bootstrap_game_servers()
}

/// Starts up the game server
fn bootstrap_game_servers() {
    GameServer::start(
        get_port(),
        map_manager::FileSystemMapManager::new(get_maps_dir()),
    );
}

fn get_maps_dir() -> String {
    std::env::var("IPG_MAPS_DIR").expect("The IPG_MAPS_DIR environment variable must be set.")
}

fn get_port() -> u16 {
    std::env::var("IPG_PORT")
        .expect("The IPG_PORT environment variable must be set.")
        .parse::<u16>()
        .expect("The IPG_PORT enironment variable must be a number")
}
