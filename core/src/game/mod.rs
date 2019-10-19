pub mod map;
use std::sync::Arc;
use std::future::Future;
use std::collections::VecDeque;
use rand_xoshiro::Xoshiro128StarStar;
use rand_xoshiro::rand_core::SeedableRng;
use rand::RngCore;
use rand::Rng;
use std::f32::consts::PI;
use std::time::SystemTime;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub index: usize,
    pub name: String,
    // state: PlayerState
}

#[derive(Serialize, Deserialize)]
enum PlayerState {
    Waiting,
    Joined,
    Gone
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Planet {
    // If use a planet deseralized from an untrusted source, an attacker could 
    // undermine the integrety of the game by changing the planet's values
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub radius: f32,
    pub multiplier: f32,
    pub value: f32,
    pub possession: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Move {
    pub from: Planet,
    pub to: Planet,
    pub armada_size: u32,
    pub time: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Galaxy {
    pub time: u32, //?
    pub planets: Vec<Planet>,
    pub moves: Vec<Move>
}

//#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    pub map: map::Map,
    pub state: Option<Galaxy>,
    pub players: Vec<Player>,
    pub config: GameConfig
}
//assert_impl_all!(Game: Sync, Send);

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub min_players: u32
}

impl Game {
    pub fn new(map: map::Map, config: GameConfig) -> Game {
        Game {
            map,
            players: Vec::new(),
            state: None,
            config
        }
    }
}

type ModBuckets = VecDeque<Option<(u32,Vec<u32>)>>;

pub struct GameExecutor {
    pub start_time: u128,
    pub game: Game,
    pub event_source: GameEventSource,
    pub completed_move_idx: usize,
    // Alternative use VecDeque
    pub modification_buckets: ModBuckets,
}

pub enum GameEvent{
    Player(Arc<Player>),
    Move(Move),
    Start
}

#[derive(Default)]
pub struct GameEventSource {
    pub handlers: Vec<Box<FnMut(&GameEvent,&mut Game) -> () + Send + Sync>>
}

impl GameEventSource {
    pub fn on_event<H>(&mut self,handler: Box<H>) 
        where H: FnMut(&GameEvent,&mut Game) -> () + Send + Sync + 'static
    {
        self.handlers.push(handler);
    }

    pub fn emit_event(&mut self, event: GameEvent, game: &mut Game) {
        for handler in &mut self.handlers {
            handler(&event, game);
        }
    }
}

const TICKS_PER_SHIP: u32 = 60;
const SHIP_SPEED: f32 = 5f32;

fn get_millis() -> u128 {
    SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}

impl Move {
    pub fn start_positions<'a>(&'a self) -> impl Iterator<Item=(f32,f32)> + 'a {
        let mut rng = Xoshiro128StarStar::seed_from_u64(827_803_098);
        let radius = self.from.radius;
        let x = self.from.x as f32;
        let y = self.from.y as f32;
        (0..self.armada_size).map(move |_| {
            let a: f32 = 2f32 * PI * rng.gen::<f32>();
            let radius = rng.gen::<f32>().sqrt() * radius;
            (x as f32 + radius * a.cos(), y as f32 + radius * a.sin())
        })
    }

    fn dist(&self) -> f32 {
        (
            (self.from.x as f32 - self.to.y  as f32).powf(2.0) + 
            (self.from.y  as f32 - self.to.y  as f32).powf(2.0)
        ).sqrt()
    }

    pub fn end_time(&self) -> u32 {
        let dist = self.dist()  + self.from.radius - self.to.radius;
        (dist/SHIP_SPEED) as u32 + self.time
    }

    pub fn first_arrival_time(&self) -> u32 {
        let dist = self.dist() - self.from.radius - self.to.radius;
        (dist/SHIP_SPEED) as u32 + self.time
    }
}

impl GameExecutor {
    pub fn from_game(game: Game) -> GameExecutor {
        GameExecutor {
            start_time: 0,
            game,
            event_source: GameEventSource::default(),
            completed_move_idx: 0,
            modification_buckets: VecDeque::new()
        }
    }

    pub fn add_player(&mut self, mut player: Player) -> Result<Player,String> {
        if self.game.map.planets[0].possession.len() > self.game.players.len() {
            player.index = self.game.players.len();
            let player_cpy = player.clone();
            self.game.players.push(player);
            Ok(player_cpy)
        } else {
            Err("Game is already full.".to_owned())
        }
    }

    pub fn start_game(&mut self) -> Result<(),String> {
        if self.game.state.is_some() {
            Err("Cannot start game, it has already started.".to_owned())
        } else if (self.game.players.len() as u32) < self.game.config.min_players {
           Err("Cannot start game, insuffcient players".to_owned())
        } else {
           self.game.state = Some(self.game.map.to_galaxy(&mut self.game.players)?);
           self.start_time = get_millis();
           self.event_source.emit_event(GameEvent::Start,&mut self.game);
           Ok(())
        }
    }

    fn spawn_ships(planets: &mut Vec<Planet>, elapsed: u32) {
        for planet in planets.iter_mut() {
            planet.value += planet.multiplier * planet.radius * elapsed as f32 / TICKS_PER_SHIP as f32;
        };
    }

    #[inline(never)]
    fn process_move(time: &mut u32, planets: &mut Vec<Planet>, mod_buckets: &mut ModBuckets, game_move: &Move) {
        if  *time != game_move.time {
            panic!("Moves should only be processed on a game state that matches the move time.");
        }

        let armada = game_move.armada_size;
        planets[game_move.from.index].value -= armada as f32;
        let first_time = game_move.first_arrival_time();
        for ship_pos in game_move.start_positions() {
            let dist = ((ship_pos.0 - game_move.to.x as f32).powf(2.0) + (ship_pos.1 - game_move.to.y as f32).powf(2.0)).sqrt() - game_move.to.radius;
            // Time of arrival
            let arrival = (dist/SHIP_SPEED) as u32 + game_move.time;
            // Bucket index is offset from the oldest bucket
            let first_bucket_time = mod_buckets.get(0).and_then(|o| o.as_ref()).map(|v|v.0).unwrap_or(first_time);
            let bucket_idx = (arrival - first_bucket_time) as usize;
            let cap = ((arrival - first_bucket_time) as usize + 1).max(mod_buckets.len());
            mod_buckets.resize(cap,None);
            if let Some(ref mut bucket) = mod_buckets[bucket_idx]  {
                bucket.1[game_move.to.index] += 1;
            } else {
                mod_buckets[bucket_idx] = Some((arrival,{ 
                    let mut vec = vec![0; planets.len()];
                    vec[game_move.to.index] += 1;
                    vec
                }));
            };
        }
    }

    fn apply_buckets(time: &mut u32, planets: &mut Vec<Planet>, mod_buckets: &mut ModBuckets, target_time: u32) {
        let mut prev_time = *time;
        while !mod_buckets.is_empty() && mod_buckets[0].as_ref().map(|b| b.0 <= target_time).unwrap_or(true) {
            if let Some(bucket) = mod_buckets.pop_front().and_then(std::convert::identity) {
                GameExecutor::spawn_ships(planets,bucket.0 - prev_time);
                prev_time = bucket.0;
                for (i, planet) in planets.iter_mut().enumerate() {
                    planet.value -= bucket.1[i] as f32;
                }
            }
        };
        GameExecutor::spawn_ships(planets,target_time - prev_time);
    }

    pub fn step_to(&mut self, target_time: u32) {
        if let Some(ref mut galaxy) = self.game.state {
            let prev_time = galaxy.time;
            println!("{}",galaxy.time);
            let new_moves = galaxy.moves.iter().skip(self.completed_move_idx).filter(|game_move| {
                println!("result: {},prev_time: {}, target_time: {}, move.time: {}",game_move.time >= prev_time && game_move.time <= target_time, prev_time, target_time, game_move.time);
                // >= in first condition might result in double processing moves
                game_move.time >= prev_time && game_move.time <= target_time 
            });
            for game_move in new_moves {
                GameExecutor::apply_buckets(&mut galaxy.time, &mut galaxy.planets,&mut self.modification_buckets, game_move.time);
                galaxy.time = game_move.time;
                GameExecutor::process_move(&mut galaxy.time,&mut galaxy.planets,&mut self.modification_buckets, game_move);
            };
            if galaxy.time < target_time {
                GameExecutor::apply_buckets(&mut galaxy.time, &mut galaxy.planets,&mut self.modification_buckets, target_time);
            };
            galaxy.time = target_time;
        }
    }

    pub fn create_move(&mut self, from: u16, to: u16) -> Result<Move,String> {
        let time = self.get_time();
        self.step_to(time);
        let galaxy = self.game.state.as_mut().ok_or_else(|| "Game has not been started.".to_owned())?;
        Ok(Move {
            to: galaxy.planets[to as usize].clone(),
            from: galaxy.planets[from as usize].clone(),
            armada_size: galaxy.planets[from as usize].value as u32 / 2,
            time
        })
    }

    pub fn get_time(&self) -> u32 {
        ((get_millis() - self.start_time)/17) as u32
    }

    pub fn add_move(&mut self, player: &Player,game_move: Move) -> Result<(),String> {
        self.step_to(game_move.time);
        let galaxy = self.game.state.as_mut().ok_or_else(|| "Game has not been started.".to_owned())?;
        if galaxy.planets[game_move.from.index].possession.map_or(false,|idx| player.index != idx) {
            Err("Planet not owned by player.".to_owned())
        } else {
            galaxy.moves.push(game_move.clone());
            self.event_source.emit_event(GameEvent::Move(game_move), &mut self.game);
            Ok(())
        }
    }
}