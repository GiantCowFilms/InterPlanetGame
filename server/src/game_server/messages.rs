use crate::game::Galaxy
use serde::{Deserialize, Serialize};

enum MessageType {
    #[derive(Deserialize, Seralize)]
    SetName {
        name: String,
    },
    #[derive(Deserialize, Seralize)]
    EnterGame {
        game_id: String
    },
    #[derive(Deserialize, Seralize)]
    GameState {
        galaxy: Galaxy
    },
    #[derive(Deserialize, Seralize)]
    GameMove {
        to: u16
        from: u16
    },
    TimedGameMove {
        time: u64,
        game_move: MessageType::GameMove
    },
    #[derive(Deserialize, Seralize)]
    ExitGame {

    }
    #[derive(Deserialize, Seralize)]
    CreateGame {
        map_id: string
    }
}