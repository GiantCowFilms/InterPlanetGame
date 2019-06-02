use crate::game::Galaxy;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SetName {
    name: String
}

#[derive(Deserialize, Serialize)]
pub struct EnterGame {
    pub game_id: String,
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

#[derive(Deserialize, Serialize)]
pub struct GameMetadata {
    pub game_id: String
}

#[derive(Deserialize, Serialize)]
pub struct GameList {
    pub games: Vec<GameMetadata>
}

#[derive(Deserialize, Serialize)]
pub enum MessageType {
    SetName(SetName),
    EnterGame(EnterGame),
    GameState(GameState),
    GameMove(GameMove),
    TimedGameMove(TimedGameMove),
    ExitGame,
    NewGame(GameMetadata),
    GameList(GameList),
    CreateGame(CreateGame),
    Error(String)
}