pub mod map;
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
pub struct Player {
    name: String,
    state: PlayerState
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
    possession: Option<Rc<Player>>,
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

#[derive(Serialize, Deserialize)]
pub struct Game {
    map: map::Map,
    state: Galaxy,
    players: Vec<Player>
}

impl Game {
    fn from_map(map: map::Map) {
        
    }
}