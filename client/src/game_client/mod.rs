use ipg_core::game::{Game, GameExecutor, Planet, Player, map::Map};
use ipg_core::protocol::messages::{
    GameList, GameMetadata, GameMove, GameState, MessageType, SetName,
};
use js_sys;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebSocket, CanvasRenderingContext2d , Window};
mod game_render;
use self::game_render::GameRender;

#[wasm_bindgen]
pub struct GameClient {
    game_list: Vec<GameMetadata>,
    // on_game_list: Vec<Box<Fn () -> () + 'static>>,
    current_game_state: Option<GameExecutor>,
    current_game: Option<GameMetadata>,
    current_game_render: Option<GameRender>,
    current_game_players: Option<Vec<Player>>,
    selected_planet: Option<Planet>, //Todo maybe move this somewhere else?
    current_posession_index: Option<u32>,
    socket: WebSocket,
    maps: HashMap<String,Map>
}

#[wasm_bindgen]
impl GameClient {
    pub fn new(socket: WebSocket) -> GameClient {
        GameClient {
            game_list: Vec::new(),
            current_game: None,
            current_game_state: None,
            current_game_render: None,
            current_game_players: None,
            selected_planet: None,
            current_posession_index: None,
            socket, // on_game_list: Vec::new()
            maps: HashMap::new()
        }
    }

    pub fn handle_message(&mut self, msg_body: String) -> Option<String> {
        log!("{}", msg_body.as_str());
        let message = serde_json::from_str::<MessageType>(msg_body.as_str()).unwrap();
        match message {
            MessageType::NewGame(game_metadata) => {
                self.game_list.push(game_metadata);
                Some("NewGame".to_string())
            }
            MessageType::RemoveGame(game_id) => {
                self.game_list.retain(|game_exec| {
                    return game_exec.game_id != game_id;
                });
                Some("GameList".to_string())
            },
            MessageType::GameList(GameList { games }) => {
                self.game_list = games;
                Some("GameList".to_string())
            }
            MessageType::GameState(GameState { galaxy }) => {
                if let Some(ref mut exec) = self.current_game_state {
                    // TODO move this into setter
                    log!("galaxy state with time = {}",galaxy.time);
                    exec.game.state = Some(galaxy);
                }
                Some("GameState".to_string())
            }
            MessageType::Possession(possession) => {
                self.current_posession_index = Some(possession);
                Some("Possesion".to_string())
            }
            MessageType::Game(game) => {
                match &mut self.current_game_state {
                    Some(exec) => exec.set_game(game),
                    None => if let Some(game_metadata) = &self.current_game { 
                        self.current_game_state = Some(GameExecutor::from_game(game,game_metadata.game_id.clone()))
                    } else {
                        panic!("Attempted to load game state when no game is joined!");
                    },
                };
                Some("Game".to_string())
            }
            MessageType::MapList(map_list) => {
                self.maps = map_list;
                Some("MapList".to_owned())
            },
            MessageType::GamePlayers(players) => {
                if let Some(executor) = self.current_game_state.as_mut() {
                    executor.game.players = players;
                } else {
                    self.current_game_players = Some(players);
                };
                Some("GamePlayers".to_owned())
            },
            MessageType::Time(time) => {
                Some("Time".to_owned())
            },
            _ => None,
        }
    }

    /// Gets the complete list of games that the server is hosting.
    pub fn game_list(&self) -> JsValue {
        JsValue::from_serde(&self.game_list).unwrap()
    }
    // pub fn game_list(&self) -> Box<[GameMetadata]> {
    //     self.game_list.clone().into_boxed_slice()
    // }

    pub fn create_game(&self) {}

    pub fn set_name(&self, name: String) {
        let message = serde_json::to_string(&MessageType::SetName(SetName { name })).unwrap();
        self.socket.send_with_str(message.as_str());
    }

    /// Returns the current game time
    pub fn get_time(&self) -> Option<u32> {
        self.current_game_state
            .as_ref()
            .and_then(|state| state.game.state.as_ref().map(|s| s.time))
    }

    // pub fn get_start_time(&self) -> Option<u32> {
    //     self.current_game_state
    //         .as_ref()
    //         .and_then(|state| state.game.state.as_ref().map(|s| s.start_time))
    // }

    pub fn get_clock_offset(&self) {
        let window = web_sys::window().expect("Should have a window in this context");
        let time = window.performance().expect("Unable to access performance").now() as u128;
        let message = serde_json::to_string(&MessageType::Time(0))
        .unwrap();
        self.socket.send_with_str(message.as_str());
    }

    pub fn enter_game(&mut self, game_metadata: JsValue) {
        if let Ok(game_metadata) =
            game_metadata.into_serde() as Result<GameMetadata, serde_json::Error>
        {
            let message = serde_json::to_string(&MessageType::EnterGame(game_metadata.game_id.to_owned()))
            .unwrap();
            self.socket.send_with_str(message.as_str());
            self.current_game = Some(game_metadata);
        }
    }

    pub fn start_game(&self) -> Result<(), JsValue> {
        let message = serde_json::to_string(&MessageType::StartGame).unwrap();
        self.socket.send_with_str(message.as_str())
    }

    pub fn set_render_target(
        &mut self,
        canvas_top: HtmlCanvasElement,
        canvas_bottom: HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        self.current_game_render = Some(GameRender::new(canvas_top, canvas_bottom)?);
        Ok(())
    }

    pub fn preview_game(&self, canvas: &HtmlCanvasElement, map_id: String) -> Result<(),JsValue> {
        let ctx_2d = canvas.get_context("2d")?.expect("Unwrap 2d context")
            .dyn_into::<CanvasRenderingContext2d>()?;
        let map = self.maps.get(&map_id).ok_or(format!("Map {} not found.",map_id))?;
        self::game_render::render_map(&ctx_2d,map,2,canvas.width(),canvas.height());
        Ok(())
    }

    pub fn get_maps(&self) -> js_sys::Array {
        self.maps.keys().map(|k| JsValue::from(k)).fold(js_sys::Array::new(),|arr,v| {
            arr.push(&v);
            arr
        })
    }

    pub fn get_player_list(&self) -> Option<js_sys::Array> {
        self.current_game_state.as_ref().map(|state| {
            &state.game.players
        }).or(self.current_game_players.as_ref()).map(|players| {
            players.iter().map(|k| JsValue::from_serde(k).unwrap()).fold(js_sys::Array::new(),|arr,v| {
                arr.push(&v);
                arr
            })
        })
    }

    pub fn render_game_frame(&mut self, mut time: u32) -> Result<(), JsValue> {
        let exec = self
            .current_game_state
            .as_mut()
            .ok_or("No game state loaded.")
            .map_err(|err| JsValue::from(err))?;
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

    pub fn mouse_event(&mut self, x: f32, y: f32) -> Result<(), JsValue> {
        let galaxy = self
            .current_game_state
            .as_ref()
            .and_then(|exec| exec.game.state.as_ref())
            .ok_or("No game state loaded.")
            .map_err(|err| JsValue::from(err))?;
        let mut selected_planet = None;
        for planet in &galaxy.planets {
            if planet.radius.powf(2f32)
                > (planet.x as f32 - x).powf(2f32) + (planet.y as f32 - y).powf(2f32)
            {
                selected_planet = Some(planet);
                break;
            }
        }
        format!("{}", selected_planet.is_none());
        if let Some(selected_planet) = selected_planet {
            if let Some(source_planet) = &self.selected_planet {
                self.make_move(&source_planet, selected_planet);
                self.selected_planet = None;
            } else if selected_planet
                .possession
                .map(|p| Some(p as u32) == self.current_posession_index)
                .unwrap_or(false)
            {
                self.selected_planet = Some(selected_planet.clone());
            }
        }
        Ok(())
    }

    fn make_move(&self, from: &Planet, to: &Planet) {
        let message = serde_json::to_string(&MessageType::GameMove(GameMove {
            to: to.index as u16,
            from: from.index as u16,
        }))
        .unwrap();
        self.socket.send_with_str(message.as_str());
    }
}