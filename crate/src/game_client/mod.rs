use ipg_core::protocol::messages::{ MessageType , GameMetadata, GameList };
use ipg_core::game::Game;
use wasm_bindgen::prelude::*;
use js_sys;

#[wasm_bindgen]
#[derive(Default)]
pub struct GameClient{
    game_list: Vec<GameMetadata>,
    // on_game_list: Vec<Box<Fn () -> () + 'static>>,
    current_game_state: Option<Game>,
    current_game: Option<GameMetadata>,
}

#[wasm_bindgen]
impl GameClient {
    pub fn new () -> GameClient {
        GameClient {
            game_list: Vec::new(),
            current_game: None,
            current_game_state: None,
            // on_game_list: Vec::new()
        }
    }
    pub fn handle_message(&mut self, msg_body: String) -> Option<String> {
        log!("{}",msg_body.as_str());
        let message = serde_json::from_str::<MessageType>(msg_body.as_str()).unwrap();
        match message {
            MessageType::NewGame(game_metadata) => {
                self.game_list.push(game_metadata);
                Some("GameList".to_string())
            },
            MessageType::GameList(GameList {
                games
            }) => {
                self.game_list = games;
                Some("GameList".to_string())
            },
            _ => None
        }
    }

    pub fn game_list(&self) -> JsValue {
        JsValue::from_serde(&self.game_list).unwrap()
    }

    pub fn create_game(&self) {
        
    }

    // /// Calls a callback when the game_list changes.
    // /// 
    // pub fn on_game_list(&mut self, handler: js_sys::Function) {
    //     self.on_game_list.push(Box::new(move || {
    //         handler.call1(&JsValue::NULL, &JsValue::from_serde(&self.game_list).unwrap());
    //     }));
    // }
}