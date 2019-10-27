use ipg_core::protocol::messages::{ MessageType , GameMetadata, GameList, GameState, SetName , GameMove};
use ipg_core::game::{ Game, Planet, GameExecutor };
use wasm_bindgen::prelude::*;
use js_sys;
use web_sys::{WebSocket, HtmlCanvasElement};
mod game_render;
use self::game_render::GameRender;

#[wasm_bindgen]
pub struct GameClient{
    game_list: Vec<GameMetadata>,
    // on_game_list: Vec<Box<Fn () -> () + 'static>>,
    current_game_state: Option<GameExecutor>,
    current_game: Option<GameMetadata>,
    current_game_render: Option<GameRender>,
    selected_planet: Option<Planet>, //Todo maybe move this somewhere else?
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
            selected_planet: None,
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
                if let Some(ref mut exec) = self.current_game_state {
                    // TODO move this into setter
                    exec.game.state = Some(galaxy);
                }
                Some("GameState".to_string())
            },
            MessageType::Game(game) => {
                match &mut self.current_game_state {
                    Some(exec) => exec.set_game(game),
                    None => { 
                        self.current_game_state = Some(GameExecutor::from_game(game)) 
                    }
                };
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

    pub fn get_time(&self) -> Option<u32> {
        self.current_game_state.as_ref().and_then(|state| {
            state.game.state.as_ref().map(|s| s.time)
        })
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

    pub fn render_game_frame(&mut self, mut time: u32) -> Result<(), JsValue> {
        let exec = self.current_game_state.as_mut().ok_or("No game state loaded.").map_err(|err| JsValue::from(err))?;
        if let Some(ref galaxy) = exec.game.state {
            // temprorary in lieu of proper sync
            time = galaxy.time.max(time);
            if time < galaxy.time {
                return Err(JsValue::from("Cannot render frames from the past."));
            };
            exec.step_to(time);
        }
        if let Some(render) = &mut self.current_game_render {
            render.render_galaxy(&exec.game)?;
        };
        Ok(())
    }

    pub fn mouse_event(&mut self, x: f32, y: f32) -> Result<(),JsValue> {
        let galaxy = self.current_game_state.as_ref().and_then(|exec| { exec.game.state.as_ref() }).ok_or("No game state loaded.").map_err(|err| JsValue::from(err))?;
        let mut selected_planet = None;
        for planet in &galaxy.planets {
            if planet.radius.powf(2f32) > (planet.x as f32 - x).powf(2f32) + (planet.y as f32 - y).powf(2f32) {
                selected_planet = Some(planet);
                break;
            }
        };
        format!("{}",selected_planet.is_none());
        if let Some(selected_planet) = selected_planet {
            if let Some(source_planet) = &self.selected_planet {
                self.make_move(&source_planet,selected_planet);
                self.selected_planet = None;
            } else if selected_planet.possession.map(|p| p == 1).unwrap_or(false) {
                self.selected_planet = Some(selected_planet.clone());
            }
        }
        Ok(())
    }

    fn make_move(&self, from: &Planet, to: &Planet) {
        let message = serde_json::to_string(&MessageType::GameMove(GameMove {
            to: to.index as u16,
            from: from.index as u16
        })).unwrap();
        self.socket.send_with_str(message.as_str());
    }  
}