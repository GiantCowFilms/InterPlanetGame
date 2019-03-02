pub mod map;
use std::rc::Rc;

pub struct Player {
    name: String,
    state: PlayerState
}

enum PlayerState {
    Waiting,
    Joined,
    Gone
}

pub struct Planet {
    x: u32,
    y: u32,
    radius: u32,
    multiplier: f64,
    value: u32,
    possession: Option<Rc<Player>>,
}

pub struct Move {
    from: Planet,
    to: Planet
}

pub struct Galaxy {
    time: u32, //?
    planets: Vec<Planet>,
    moves: Vec<Move>
}

pub struct Game {
    map: map::Map,
    state: Galaxy,
    players: Vec<Player>
}

impl Game {
    fn from_map(map: map::Map) {
        
    }
}