use server::game_server::messages::{ MessageType , GameMetadata, GameList };
use server::game::Game;

#[wasm_bindgen]
pub struct GameClient {
    game_list: Vec<GameMetadata>,
    currentGameState: Option<Game>,
    currentGame: Option<GameMetadata>,
}

impl GameClient {
    fn handle_message(&mut self, msg_body: String) {
        let message = serde_json::from_str::<MessageType>(msg_body.as_str());
        match message {
            NewGame(game_metadata) => {
                self.game_list.push(game_metadata)
            },
            GameList(GameList {
                games
            }) => {
                self.game_list = games;
            },
        }
    }
}