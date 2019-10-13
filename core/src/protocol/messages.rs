use crate::game::Galaxy;
use serde::{Deserialize, Serialize};

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    }
}

#[derive(Deserialize, Serialize)]
pub struct SetName {
    pub name: String
}

#[derive(Deserialize,Serialize)]
pub struct GameState {
    galaxy: Galaxy
}

#[derive(Deserialize, Serialize)]
pub struct GameMove {
    to: u16,
    from: u16
}

#[derive(Deserialize, Serialize)]
pub struct TimedGameMove {
    time: u64,
    game_move: GameMove
}

#[derive(Deserialize, Serialize)]
pub struct CreateGame {
    pub map_id: String
}

//#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Deserialize, Serialize)]
pub struct GameMetadata {
    pub game_id: String
}

#[derive(Deserialize, Serialize)]
pub struct GameList {
    pub games: Vec<GameMetadata>
}

#[derive(Deserialize, Serialize)]
pub struct GamePlayers {
    pub game_id: String,
    pub players: Vec<PlayerMetadata>
}

#[derive(Deserialize, Serialize)]
pub struct PlayerMetadata {
    name: String
}

#[derive(Deserialize, Serialize)]
pub enum MessageType {
    SetName(SetName),
    EnterGame(GameMetadata),
    GameState(GameState),
    GameMove(GameMove),
    GamePlayers(GamePlayers),
    TimedGameMove(TimedGameMove),
    ExitGame,
    NewGame(GameMetadata),
    GameList(GameList),
    CreateGame(CreateGame),
    Error(String)
}