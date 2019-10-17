use ipg_core::protocol::messages::{ MessageType , GameMetadata, GameList, GameState, SetName };
use ipg_core::game::Game;
use wasm_bindgen::prelude::*;
use js_sys;
use web_sys::{WebSocket, HtmlCanvasElement};
mod game_render;
use self::game_render::GameRender;

#[wasm_bindgen]
pub struct GameClient{
    game_list: Vec<GameMetadata>,
    // on_game_list: Vec<Box<Fn () -> () + 'static>>,
    current_game_state: Option<Game>,
    current_game: Option<GameMetadata>,
    current_game_render: Option<GameRender>,
    socket: WebSocket
}

#[wasm_bindgen]
impl GameClient {
    pub fn new (socket: WebSocket) -> GameClient {
        GameClient {
            game_list: Vec::new(),
            current_game: None,
            current_game_state: None,
            current_game_render: None,
            socket
            // on_game_list: Vec::new()
        }
    }

    pub fn handle_message(&mut self, msg_body: String) -> Option<String> {
        log!("{}",msg_body.as_str());
        let message = serde_json::from_str::<MessageType>(msg_body.as_str()).unwrap();
        match message {
            MessageType::NewGame(game_metadata) => {
                self.game_list.push(game_metadata);
                Some("NewGame".to_string())
            },
            MessageType::GameList(GameList {
                games
            }) => {
                self.game_list = games;
                Some("GameList".to_string())
            },
            MessageType::GameState(GameState {
                galaxy
            }) => {
                if let Some(ref mut game) = self.current_game_state {
                    game.state = Some(galaxy);
                }
                Some("GameState".to_string())
            },
            MessageType::Game(game) => {
                self.current_game_state = Some(game);
                Some("Game".to_string())
            }
            _ => None
        }
    }

    /// Gets the complete list of games that the server is hosting.
    pub fn game_list(&self) -> JsValue {
        JsValue::from_serde(&self.game_list).unwrap()
    }
    // pub fn game_list(&self) -> Box<[GameMetadata]> {
    //     self.game_list.clone().into_boxed_slice()
    // }

    pub fn create_game(&self) {

    }

    pub fn set_name(&self, name: String) {
        let message = serde_json::to_string(&MessageType::SetName(SetName {
            name
        })).unwrap();
        self.socket.send_with_str(message.as_str());
    }

    pub fn enter_game(&self,game_metadata: JsValue) {
        if let Ok(game_metadata) = game_metadata.into_serde() as Result<GameMetadata,serde_json::Error>  {
            let message = serde_json::to_string(&MessageType::EnterGame(GameMetadata {
                game_id: game_metadata.game_id.to_owned()
            })).unwrap();
            self.socket.send_with_str(message.as_str());
        }
    }

    pub fn start_game(&self) -> Result<(),JsValue> {
        let message = serde_json::to_string(&MessageType::StartGame).unwrap();
        self.socket.send_with_str(message.as_str())
    }
    
    pub fn set_render_target(&mut self, canvas_top: HtmlCanvasElement, canvas_bottom: HtmlCanvasElement) -> Result<(),JsValue> {
        self.current_game_render = Some(GameRender::new(canvas_top,canvas_bottom)?);
        Ok(())
    }

    pub fn render_game_frame(&mut self, time: u32) -> Result<(), JsValue> {
        let game = self.current_game_state.as_ref().ok_or("No game state loaded.").map_err(|err| JsValue::from(err))?;
        if let Some(ref galaxy) = game.state {
            if time < galaxy.time {
                return Err(JsValue::from("Cannot render frames from the past."));
            };
        }
        if let Some(render) = &mut self.current_game_render {
            render.render_galaxy(&game);
        };
        Ok(())
    }
}