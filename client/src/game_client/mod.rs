use ipg_core::game::{map::Map, GameExecutor, Planet, Player};
use ipg_core::protocol::messages::{
    GameList, GameMetadata, GameMove, GameState, MessageType, SetName,
};
use js_sys;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WebSocket};
mod game_render;
use self::game_render::GameRender;

struct JoinedGame {
    exec: GameExecutor,
    #[allow(unused)]
    metadata: GameMetadata,
    render: GameRender,
    possesion_index: u32,
    selected_planet: Option<Planet>,
}
struct Waiting {
    metadata: GameMetadata,
    render: GameRender,
    possesion_index: Option<u32>,
    players: Option<Vec<Player>>,
}

enum ActiveGame {
    Waiting(Waiting),
    Joined(JoinedGame),
    None,
}

impl ActiveGame {
    pub fn joined(&self) -> Option<&JoinedGame> {
        match self {
            ActiveGame::Joined(game) => Some(game),
            _ => None,
        }
    }
    #[allow(unused)]
    pub fn joined_mut(&mut self) -> Option<&mut JoinedGame> {
        match self {
            ActiveGame::Joined(game) => Some(game),
            _ => None,
        }
    }
}

#[wasm_bindgen]
pub struct GameClient {
    game_list: Vec<GameMetadata>,
    current_game: ActiveGame,
    socket: WebSocket,
    maps: HashMap<String, Map>,
}

#[wasm_bindgen]
impl GameClient {
    pub fn new(socket: WebSocket) -> GameClient {
        GameClient {
            game_list: Vec::new(),
            current_game: ActiveGame::None,
            socket, // on_game_list: Vec::new()
            maps: HashMap::new(),
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
            }
            MessageType::GameList(GameList { games }) => {
                self.game_list = games;
                Some("GameList".to_string())
            }
            MessageType::Time(_time) => Some("Time".to_owned()),
            MessageType::MapList(map_list) => {
                self.maps = map_list;
                Some("MapList".to_owned())
            }
            MessageType::EnterGame(_game_id) => Some("EnterGame".to_owned()),
            message => {
                match self.current_game {
                    ActiveGame::Joined(ref mut current) => {
                        match message {
                            MessageType::GameState(GameState { galaxy }) => {
                                // TODO move this into setter
                                log!("galaxy state with time = {}", galaxy.time);
                                current.exec.game.state = Some(galaxy);
                                Some("GameState".to_string())
                            }
                            MessageType::Possession(possession) => {
                                current.possesion_index = possession;
                                Some("Possesion".to_string())
                            }
                            MessageType::Game(game) => {
                                current.exec.set_game(game);
                                Some("Game".to_string())
                            }
                            MessageType::GamePlayers(players) => {
                                current.exec.game.players = players;
                                Some("GamePlayers".to_owned())
                            }
                            _ => None,
                        }
                    }
                    ActiveGame::Waiting(ref mut waiting) => {
                        match message {
                            MessageType::Game(game) => {
                                let owned =
                                    std::mem::replace(&mut self.current_game, ActiveGame::None);
                                let waiting = if let ActiveGame::Waiting(waiting) = owned {
                                    waiting
                                } else {
                                    unreachable!();
                                };
                                self.current_game = ActiveGame::Joined(JoinedGame {
                                    exec: GameExecutor::from_game(
                                        game,
                                        waiting.metadata.game_id.clone(),
                                    ),
                                    metadata: waiting.metadata,
                                    render: waiting.render,
                                    possesion_index: waiting.possesion_index.expect(
                                        "Cannot start game until posession index has been sent",
                                    ),
                                    selected_planet: None,
                                });
                                Some("Game".to_string())
                            }
                            _ => {
                                match message {
                                    MessageType::Possession(possession) => {
                                        waiting.possesion_index = Some(possession);
                                        Some("Possesion".to_string())
                                    }
                                    MessageType::GamePlayers(players) => {
                                        waiting.players = Some(players);
                                        Some("GamePlayers".to_owned())
                                    }
                                    _ => {
                                        // We should only get here for messages which
                                        // are meant to be handled by the Joining state.

                                        // Commented out for now because there are un-handled messages that are incorrectly arriving at these branches.
                                        // panic!("recieved game state while joining, or standard message handler is missing.");
                                        None
                                    }
                                }
                            }
                        }
                    }
                    ActiveGame::None => {
                        // See commented panic above
                        // panic!("recieved game state message while outside of game!");
                        None
                    }
                }
            }
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
        let _ = self.socket.send_with_str(message.as_str());
    }

    /// Returns the current game time
    pub fn get_time(&self) -> Option<u32> {
        self.current_game
            .joined()
            .as_ref()
            .and_then(|joined| joined.exec.game.state.as_ref().map(|s| s.time))
    }

    // pub fn get_start_time(&self) -> Option<u32> {
    //     self.current_game_state
    //         .as_ref()
    //         .and_then(|state| state.game.state.as_ref().map(|s| s.start_time))
    // }

    pub fn get_clock_offset(&self) {
        let window = web_sys::window().expect("Should have a window in this context");
        let _time = window
            .performance()
            .expect("Unable to access performance")
            .now() as u128;
        let message = serde_json::to_string(&MessageType::Time(0)).unwrap();
        let _ = self.socket.send_with_str(message.as_str());
    }

    pub fn enter_game(
        &mut self,
        game_metadata: JsValue,
        // HTML pointers needed to render the game
        canvas_top: HtmlCanvasElement,
        canvas_bottom: HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        if let Ok(game_metadata) =
            game_metadata.into_serde() as Result<GameMetadata, serde_json::Error>
        {
            let message =
                serde_json::to_string(&MessageType::EnterGame(game_metadata.game_id.to_owned()))
                    .unwrap();
            let _ = self.socket.send_with_str(message.as_str());
            self.current_game = ActiveGame::Waiting(Waiting {
                metadata: game_metadata,
                players: None,
                possesion_index: None,
                render: GameRender::new(canvas_top, canvas_bottom)?,
            });
        }
        Ok(())
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
        match self.current_game {
            ActiveGame::Joined(ref mut game) => {
                game.render = GameRender::new(canvas_top, canvas_bottom)?;
            }
            ActiveGame::Waiting(ref mut waiting) => {
                waiting.render = GameRender::new(canvas_top, canvas_bottom)?;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn preview_game(&self, canvas: &HtmlCanvasElement, map_id: String) -> Result<(), JsValue> {
        let ctx_2d = canvas
            .get_context("2d")?
            .expect("Unwrap 2d context")
            .dyn_into::<CanvasRenderingContext2d>()?;
        let map = self
            .maps
            .get(&map_id)
            .ok_or(format!("Map {} not found.", map_id))?;
        self::game_render::render_map(&ctx_2d, map, 2, canvas.width(), canvas.height())?;
        Ok(())
    }

    pub fn get_maps(&self) -> js_sys::Array {
        self.maps
            .keys()
            .map(|k| JsValue::from(k))
            .fold(js_sys::Array::new(), |arr, v| {
                arr.push(&v);
                arr
            })
    }

    pub fn get_player_list(&self) -> Option<js_sys::Array> {
        match &self.current_game {
            ActiveGame::Joined(current) => Some(&current.exec.game.players),
            ActiveGame::Waiting(Waiting { players, .. }) => players.as_ref(),
            _ => None,
        }
        .map(|players| {
            players
                .iter()
                .map(|k| JsValue::from_serde(k).unwrap())
                .fold(js_sys::Array::new(), |arr, v| {
                    arr.push(&v);
                    arr
                })
        })
    }

    pub fn render_game_frame(&mut self, mut time: u32) -> Result<(), JsValue> {
        if let ActiveGame::Joined(current) = &mut self.current_game {
            if let Some(ref galaxy) = current.exec.game.state {
                // temprorary in lieu of proper sync
                time = galaxy.time.max(time);
                if time < galaxy.time {
                    return Err(JsValue::from("Cannot render frames from the past."));
                };
                current.exec.step_to(time);
            }
            if let ActiveGame::Joined(current) = &mut self.current_game {
                current.render.render_galaxy(&current.exec.game)?;
            };
            Ok(())
        } else {
            Err("No game state loaded.".into())
        }
    }

    pub fn mouse_event(&mut self, x: f32, y: f32) -> Result<(), JsValue> {
        if let ActiveGame::Joined(ref game) = self.current_game {
            let galaxy = game
                .exec
                .game
                .state
                .as_ref()
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
            if let Some(selected_planet) = selected_planet {
                if let Some(source_planet) = &game.selected_planet {
                    self.make_move(&source_planet, selected_planet);
                    if let ActiveGame::Joined(ref mut game) = self.current_game {
                        game.selected_planet = None
                    }
                } else if selected_planet
                    .possession
                    .map(|p| p as u32 == game.possesion_index)
                    .unwrap_or(false)
                {
                    let new_selection = selected_planet.clone();
                    if let ActiveGame::Joined(ref mut game) = self.current_game {
                        game.selected_planet = Some(new_selection)
                    }
                };
            };
        } else {
            return Err(Into::<JsValue>::into("not currently in a game"));
        }
        Ok(())
    }

    fn make_move(&self, from: &Planet, to: &Planet) {
        let message = serde_json::to_string(&MessageType::GameMove(GameMove {
            to: to.index as u16,
            from: from.index as u16,
        }))
        .unwrap();
        let _ = self.socket.send_with_str(message.as_str());
    }
}
