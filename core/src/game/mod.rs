pub mod map;
use std::sync::Arc;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    // state: PlayerState
}

#[derive(Serialize, Deserialize)]
enum PlayerState {
    Waiting,
    Joined,
    Gone
}

#[derive(Serialize, Deserialize)]
pub struct Planet {
    x: u32,
    y: u32,
    radius: u32,
    multiplier: f64,
    value: u32,
    possession: Option<Arc<Player>>,
}

#[derive(Serialize, Deserialize)]
pub struct Move {
    from: Planet,
    to: Planet
}

#[derive(Serialize, Deserialize)]
pub struct Galaxy {
    time: u32, //?
    planets: Vec<Planet>,
    moves: Vec<Move>
}

//#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize)]
pub struct Game {
    map: map::Map,
    state: Option<Galaxy>,
    pub players: Vec<Arc<Player>>
}

impl Game {
    pub fn from_map(map: map::Map) -> Game {
        Game {
            map,
            players: Vec::new(),
            state: None
        }
    }
}