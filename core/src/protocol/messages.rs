use crate::game::{Galaxy, Game, GameConfig, Move, Player, map::Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    }
}

#[derive(Deserialize, Serialize)]
pub struct SetName {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct GameState {
    pub galaxy: Galaxy,
}

#[derive(Deserialize, Serialize)]
pub struct GameMove {
    pub to: u16,
    pub from: u16,
}

#[derive(Deserialize, Serialize)]
pub struct CreateGame {
    pub map_id: String,
    pub config: GameConfig,
}

//#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Deserialize, Serialize)]
pub struct GameMetadata {
    pub game_id: String,
    pub map_id: String
}

#[derive(Deserialize, Serialize)]
pub struct GameList {
    pub games: Vec<GameMetadata>,
}

#[derive(Deserialize, Serialize)]
pub struct GamePlayers {
    pub game_id: String,
    pub players: Vec<PlayerMetadata>,
}

#[derive(Deserialize, Serialize)]
pub struct PlayerMetadata {
    name: String,
}

type GameID = String;

#[derive(Deserialize, Serialize)]
pub enum MessageType {
    SetName(SetName),
    EnterGame(GameID),
    Possession(u32),
    Game(Game),
    GameState(GameState),
    GameMove(GameMove),
    GamePlayers(Vec<Player>),
    TimedGameMove(Move),
    StartGame,
    ExitGame,
    NewGame(GameMetadata),
    GameList(GameList),
    MapList(HashMap<String,Map>),
    CreateGame(CreateGame),
    Error(String),
}
