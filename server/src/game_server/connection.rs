use ipg_core::protocol::messages::{ MessageType, GameMetadata, GameState};
use crate::GameServer;
use ipg_core::game::Player;
use ipg_core::game::{ Game , GameExecutor};
use crate::game_server::GameList;
use std::sync::{ Arc, Mutex };
use futures::sink::Sink;
use futures::sync::mpsc::SendError;
use tokio_tungstenite::tungstenite::{Error, Message};
use std::future::Future;

pub trait Captures<'a> {}

impl<'a, T> Captures<'a> for T {}

pub struct GameConnection {
    player: Option<Arc<Player>>,
    current_game: Option<Arc<Mutex<GameExecutor>>>
}

impl GameConnection {
    pub fn new() -> Self {
        GameConnection {
            player: None,
            current_game: None
        }
    }

    pub fn handle_message<'a: 'd, 'b: 'd, 'c: 'd, 'd>(
        &'c mut self,
        instance: Arc<GameServer>,
        message: &'a Message,
        sink: &'b mut ((Sink<SinkItem = Message, SinkError = Error>) + Send),
    ) -> impl Future<Output = ()> + Captures<'a> + Captures<'b> + Captures<'c> + 'd {
        async move {
            let result = if let Ok(message_body) = message.to_text() {
                if let Ok(message_data) = serde_json::from_str::<MessageType>(message_body) {
                    match message_data {
                        MessageType::CreateGame(game_settings) => {
                            match (*instance).map_manager.lock() {
                                Ok(maps) => { 
                                    if let Some(map) = maps.map_by_id(&game_settings.map_id) {
                                        let game = Game::new((*map).clone(),game_settings.config);
                                        match (*instance).games.write() {
                                            Ok(mut games) => {
                                                let game_id = games.add_game(game);
                                                let seralized = serde_json::to_string(&MessageType::NewGame(GameMetadata {
                                                    game_id
                                                }));
                                                let _ = sink.start_send(Message::from(seralized.unwrap()));
                                                Ok(())
                                            }
                                            _=> Err("Game state corrupted by poisoned mutex. Please report this bug.".to_string())
                                        }
                                    } else {
                                        Err(format!("Map with id \"{}\" not found.", game_settings.map_id))
                                    }
                                }
                                Err(_poisoned) => Err("The game state is corrupted by a poisoned mutex. Please report this bug.".to_string())
                            }
                        },
                        MessageType::ExitGame => {
                            let _ = sink.start_send(Message::from("ExitGame"));
                            Ok(())
                        },
                        MessageType::EnterGame(game_metadata) => {
                            if let Ok(mut games) = instance.games.write() {
                                if let Some(game_executor_mtx) = games.get_mut(&game_metadata.game_id) { 
                                    let mut game_executor = game_executor_mtx.lock().unwrap();
                                    if let Some(player) = &self.player {
                                        game_executor.add_player(player.clone()).unwrap();                       
                                        let seralized = serde_json::to_string(&MessageType::EnterGame(GameMetadata {
                                            game_id: game_metadata.game_id.clone()
                                        }));
                                        let _ = sink.start_send(Message::from(seralized.unwrap()));
                                        if game_executor.game.state.is_some() {
                                            let seralized = serde_json::to_string(&MessageType::Game(game_executor.game.clone()));
                                            let _ = sink.start_send(Message::from(seralized.unwrap()));
                                        };
                                        self.current_game = Some(game_executor_mtx.clone());
                                        // Subscribe to game state
                                        Ok(())
                                    } else {
                                        Err("Players must set a name before joining a game.".to_string())
                                    }
                                } else {
                                    Err(format!("Could not find a game with an id of \"{}\"", &game_metadata.game_id))
                                }
                            } else {
                                Err("RwLock poisoned, game state corrupted".to_string())
                            }
                            //Send game state
                        },
                        MessageType::SetName(name_data) => {
                            //Replace player to avoid mutexes/refcells and such
                            self.player = Some(Arc::new(Player {
                                    name: name_data.name
                            }));
                            Ok(())
                        },
                        MessageType::StartGame => {
                            self.current_game.as_ref().ok_or("Player is not currently in a game".to_string()).and_then(|game_executor| {
                                game_executor.lock().map_err(|_| "Poisoned mutex".to_string())?.start_game()
                            })
                        },
                        _ => Err("The provided message type was not found.".to_string()),
                    }
                } else {
                    Err("Could not parse the provided message.".to_string())
                }
            } else {
                Err("The recieved message could not be parsed as a string.".to_string())
            };
            match result {
                Ok(_) => (),
                Err(e) => {
                    let _ = sink.start_send(Message::from(
                        serde_json::to_string(&MessageType::Error(e)).unwrap()
                    ));
                }
            };
        }
    }
}
