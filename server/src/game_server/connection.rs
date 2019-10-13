use ipg_core::protocol::messages::{ MessageType, GameMetadata, GameState};
use ipg_core::protocol::messages;
use crate::GameServer;
use ipg_core::game;
use ipg_core::game::{ Player, Game, GameExecutor, GameEvent};
use crate::game_server::GameList;
use std::sync::{ Arc, Mutex };
use futures::sink::Sink;
use futures::sync::mpsc::SendError;
use tokio_tungstenite::tungstenite::{Error, Message};
use std::future::Future;

pub trait Captures<'a> {}

impl<'a, T> Captures<'a> for T {}

pub struct GameConnection<S> 
where S: Sink<SinkItem = Message, SinkError = Error> + Send
{
    player: Option<Arc<Player>>,
    current_game: Option<Arc<Mutex<GameExecutor>>>,
    sink: Arc<Mutex<S>>,
    instance: Arc<GameServer>
}

impl<S> GameConnection<S> 
where S: Sink<SinkItem = Message, SinkError = Error> + Send + 'static
{
    pub fn new(instance: Arc<GameServer>, sink: S) -> Self {
        GameConnection {
            player: None,
            current_game: None,
            sink: Arc::new(Mutex::new(sink)),
            instance
        }
    }

    fn handle_game_event(sink: Arc<Mutex<S>>, event: &GameEvent) {
        use ipg_core::game::GameEvent::Move;
        match event {
            Start => {
                let seralized = serde_json::to_string(&MessageType::StartGame).unwrap();
                sink.lock().unwrap().start_send(Message::from(seralized));
            },
            Move(game_move) => {
                let seralized = serde_json::to_string(&MessageType::TimedGameMove(game_move.clone())).unwrap();
                sink.lock().unwrap().start_send(Message::from(seralized));
            }
            _ => {}
        }
    }

    pub fn handle_message<'a: 'c, 'b: 'c, 'c>(
        &'b mut self,
        message: &'a Message,
    ) -> impl Future<Output = ()> + Captures<'a> + Captures<'b> + 'c {
        async move {
            let mut sink = self.sink.lock().unwrap();
            let result = if let Ok(message_body) = message.to_text() {
                if let Ok(message_data) = serde_json::from_str::<MessageType>(message_body) {
                    match message_data {
                        MessageType::CreateGame(game_settings) => {
                            match self.instance.map_manager.lock() {
                                Ok(maps) => { 
                                    if let Some(map) = maps.map_by_id(&game_settings.map_id) {
                                        let game = Game::new((*map).clone(),game_settings.config);
                                        match self.instance.games.write() {
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
                            if let Ok(mut games) = self.instance.games.write() {
                                if let Some(game_executor_mtx) = games.get_mut(&game_metadata.game_id) { 
                                    let mut game_executor = game_executor_mtx.lock().unwrap();
                                    if let Some(player) = &self.player {
                                        game_executor.add_player(player.clone()).unwrap();
                                        let handler_sink = self.sink.clone();
                                        game_executor.event_source.on_event(Box::new(move |event: &GameEvent| {
                                            GameConnection::handle_game_event(handler_sink.clone(), event);
                                        }));                
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

    pub async fn handle_new_client(
        &mut self
    ) {
        use ipg_core::protocol::messages::{GameList, MessageType};
        let mut sink = self.sink.lock().unwrap();
        let result = match self.instance.games.read() {
            Ok(mut games) => {
                let games_metadata = games
                    .iter()
                    .map(|(key, val)| GameMetadata {
                        game_id: key.clone(),
                    })
                    .collect();
                let seralized = serde_json::to_string(&MessageType::GameList(GameList {
                    games: games_metadata,
                }));
                let _ = sink.start_send(Message::from(seralized.unwrap()));
                Ok(()) 
            }
            Err(_) => Err("Game state corrupted by poisned mutex.".to_string()),
        };
        if let Err(err_msg) = result {
            let _ = sink.start_send(Message::from(err_msg));
        }
    }
}