pub mod map;
use std::sync::Arc;
use std::collections::VecDeque;
use rand_xoshiro::Xoshiro128StarStar;
use rand_xoshiro::rand_core::SeedableRng;
use rand::RngCore;
use rand::Rng;

const PI: f32 = 3.141_592_653_589_793_2;

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
    // If use a planet deseralized from an untrusted source, an attacker could 
    // undermine the integrety of the game by changing the planet's values
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub radius: f32,
    pub multiplier: f32,
    pub value: f32,
    pub possession: Option<Arc<Player>>,
}

#[derive(Serialize, Deserialize)]
pub struct Move {
    from: Planet,
    to: Planet,
    time: u32
}

#[derive(Serialize, Deserialize)]
pub struct Galaxy {
    pub time: u32, //?
    pub planets: Vec<Planet>,
    pub moves: Vec<Move>
}

//#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize)]
pub struct Game {
    pub map: map::Map,
    pub state: Option<Galaxy>,
    pub players: Vec<Arc<Player>>,
}

//assert_impl_all!(Game: Sync, Send);

impl Game {
    pub fn from_map(map: map::Map) -> Game {
        Game {
            map,
            players: Vec::new(),
            state: None
        }
    }
}

type ModBuckets = VecDeque<(u32,Vec<u32>)>;

pub struct GameExecutor {
    pub game: Game,
    pub completed_move_idx: usize,
    // Alternative use VecDeque
    pub modification_buckets: ModBuckets,
}

const TICKS_PER_SHIP: u32 = 60;
const SHIP_SPEED: f32 = 5f32;

impl GameExecutor {
    pub fn from_game(game: Game) -> GameExecutor {
        GameExecutor {
            game,
            completed_move_idx: 0,
            modification_buckets: VecDeque::new()
        }
    }

    fn spawn_ships(planets: &mut Vec<Planet>, elapsed: u32) {
        for planet in planets.iter_mut() {
            planet.value += planet.multiplier * planet.radius * elapsed as f32 / TICKS_PER_SHIP as f32;
        };
    }

    fn process_move(time: &mut u32, planets: &mut Vec<Planet>, mod_buckets: &mut ModBuckets, game_move: &Move) {
        if  *time != game_move.time {
            panic!("Moves should only be processed on a game state that matches the move time.");
        }

        let armada = game_move.from.value as u32 / 2;
        planets[game_move.from.index].value -= armada as f32;
        let mut rng = Xoshiro128StarStar::seed_from_u64(827_803_098);
        for i in 0..armada {
            let a: f32 = 2f32 * PI * rng.gen::<f32>();
            let radius = rng.gen::<f32>().sqrt() * game_move.from.radius;
            let ship_pos = (radius * a.cos(), radius * a.sin());
            let dist = ((ship_pos.0 - game_move.to.x as f32).powf(2.0) + (ship_pos.1 - game_move.to.y as f32).powf(2.0)).sqrt() - game_move.to.radius;
            let arrival = (dist/SHIP_SPEED) as u32;
            let bucket_idx = arrival - mod_buckets[0].0;
            let cap = if arrival - mod_buckets[0].0 > mod_buckets.len() as u32 {
                arrival - mod_buckets[0].0 - mod_buckets.len() as u32
            } else {
                0
            };
            if mod_buckets[bucket_idx as usize].1.is_empty() {
                mod_buckets[bucket_idx as usize].1.reserve(planets.len());
            };
            mod_buckets[game_move.to.index].1[game_move.to.index] += 1;
        }
    }

    fn apply_buckets(time: &mut u32, planets: &mut Vec<Planet>, mod_buckets: &mut ModBuckets, target_time: u32) {
        let mut prev_time = *time;
        while mod_buckets[0].0 <= target_time{
            if let Some(bucket) = mod_buckets.pop_front() {
                GameExecutor::spawn_ships(planets,bucket.0 - prev_time);
                prev_time = bucket.0;
                for (i, planet) in planets.iter_mut().enumerate() {
                    planet.value -= bucket.1[i] as f32;
                }
            }
        }
    }

    pub fn step_to(&mut self, target_time: u32) {
        if let Some(ref mut galaxy) = self.game.state {
            let prev_time = galaxy.time;
            let new_moves = galaxy.moves.iter().skip(self.completed_move_idx).filter(|game_move| game_move.time > prev_time && game_move.time <= target_time);
            for game_move in new_moves {
                GameExecutor::apply_buckets(&mut galaxy.time, &mut galaxy.planets,&mut self.modification_buckets, game_move.time);
                GameExecutor::process_move(&mut galaxy.time,&mut galaxy.planets,&mut self.modification_buckets, game_move);
                galaxy.time = game_move.time;
            }
        }
    }
}