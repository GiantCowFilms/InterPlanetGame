use crate::game_server::GameList;
use crate::GameServer;
use futures::sink::Sink;
use futures::sync::mpsc::SendError;
use ipg_core::game;
use ipg_core::game::{Game, GameEvent, GameExecutor, Player};
use ipg_core::protocol::messages;
use ipg_core::protocol::messages::{GameMetadata, GameState, MessageType};
use std::future::Future;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::tungstenite::{Error, Message};

pub trait Captures<'a> {}

impl<'a, T> Captures<'a> for T {}

pub struct GameConnection<S>
where
    S: Sink<SinkItem = Message, SinkError = Error> + Send,
{
    player: Option<Player>,
    current_game: Option<Arc<Mutex<GameExecutor>>>,
    sink: Arc<Mutex<S>>,
    instance: Arc<GameServer>,
}

impl<S> GameConnection<S>
where
    S: Sink<SinkItem = Message, SinkError = Error> + Send + 'static,
{
    pub fn new(instance: Arc<GameServer>, sink: S) -> Self {
        GameConnection {
            player: None,
            current_game: None,
            sink: Arc::new(Mutex::new(sink)),
            instance,
        }
    }

    fn handle_game_event(sink: Arc<Mutex<S>>, game: &mut Game, event: &GameEvent) {
        use ipg_core::game::GameEvent::Move;
        let mut sink = sink.lock().unwrap();
        //let mut executor = executor.lock().unwrap();
        match event {
            Start => {
                let seralized = serde_json::to_string(&MessageType::StartGame).unwrap();
                sink.start_send(Message::from(seralized));
                let seralized = serde_json::to_string(&MessageType::Game(game.clone()));
                let _ = sink.start_send(Message::from(seralized.unwrap()));
            }
            Move(game_move) => {
                let seralized =
                    serde_json::to_string(&MessageType::TimedGameMove(game_move.clone())).unwrap();
                sink.start_send(Message::from(seralized));
            }
        }
    }

    pub fn handle_message<'a: 'c, 'b: 'c, 'c>(
        &'b mut self,
        message: &'a Message,
    ) -> impl Future<Output = ()> + Captures<'a> + Captures<'b> + 'c {
        async move {
            let result = || -> Result<(),String> { 
                let message_body = message.to_text().map_err(|_| "The recieved message could not be parsed as a string.".to_owned())?;
                let message_data = serde_json::from_str::<MessageType>(message_body).map_err(|_| "Could not parse the provided message.".to_owned())?;
                //Inside message handlers, always lock sinks first to avoid deadlocks
                match message_data {
                    MessageType::CreateGame(game_settings) => {
                        let mut sink = self.sink.lock().unwrap();
                        let maps = self.instance.map_manager.lock().map_err(|_| "Mutex Poisoned".to_owned())?;
                        let map = maps.map_by_id(&game_settings.map_id).ok_or_else(|| format!("Map with id \"{}\" not found.", game_settings.map_id))?;
                        let game = Game::new((*map).clone(),game_settings.config);
                        let mut games = self.instance.games.write().map_err(|_| "Game state corrupted by poisoned mutex. Please report this bug.".to_owned())?;
                        let game_id = games.add_game(game);
                        let seralized = serde_json::to_string(&MessageType::NewGame(GameMetadata {
                            game_id,
                            map_id: game_settings.map_id
                        }));
                        let _ = sink.start_send(Message::from(seralized.unwrap()));
                        Ok(())
                    }
                    MessageType::ExitGame => {
                        let mut sink = self.sink.lock().unwrap();
                        let _ = sink.start_send(Message::from("ExitGame"));
                        Ok(())
                    }
                    MessageType::EnterGame(game_id) => {
                        let mut sink = self.sink.lock().unwrap();
                        let mut games = self.instance.games.write().map_err(|_| "Game state corrupted by poisoned mutex. Please report this bug.".to_owned())?;
                        let game_executor_mtx = games.get_mut(&game_id).ok_or_else(|| format!(
                            "Could not find a game with an id of \"{}\"",
                            &game_id
                        ))?;
                        let mut game_executor = game_executor_mtx.lock().unwrap();
                        let player = self.player.as_ref().ok_or_else(|| "Players must set a name before joining a game.".to_owned())?;
                        self.player = Some(
                            game_executor
                                .add_player(player.clone())
                                .map_err(|_| "Too many players".to_owned())?
                        );
                        let handler_sink = self.sink.clone();
                        game_executor.event_source.on_event(Box::new(
                            move |event: &GameEvent, game: &mut Game| {
                                GameConnection::handle_game_event(
                                    handler_sink.clone(),
                                    game,
                                    event,
                                );
                            },
                        ));
                        let seralized = serde_json::to_string(
                            &MessageType::EnterGame(game_id.clone()),
                        );
                        let _ = sink.start_send(Message::from(seralized.unwrap()));
                        if let Some(player) = &self.player {
                            let seralized = serde_json::to_string(
                                &MessageType::Possession(player.index as u32),
                            );
                            let _ =
                                sink.start_send(Message::from(seralized.unwrap()));
                        };
                        if game_executor.game.state.is_some() {
                            let seralized = serde_json::to_string(
                                &MessageType::Game(game_executor.game.clone()),
                            );
                            let _ =
                                sink.start_send(Message::from(seralized.unwrap()));
                        };
                        self.current_game = Some(game_executor_mtx.clone());
                        // Subscribe to game state
                        Ok(())
                        //Send game state
                    }
                    MessageType::SetName(name_data) => {
                        //Replace player to avoid mutexes/refcells and such
                        self.player = Some(Player {
                            name: name_data.name,
                            index: 0, //Garbag data
                        });
                        Ok(())
                    }
                    MessageType::StartGame => self
                        .current_game
                        .as_ref()
                        .ok_or_else(|| "Player is not currently in a game".to_owned())
                        .and_then(|game_executor| {
                            game_executor
                                .lock()
                                .map_err(|_| "Poisoned mutex".to_owned())?
                                .start_game()
                        }),
                    MessageType::GameMove(game_move) => self
                        .current_game
                        .as_ref()
                        .ok_or_else(|| "Player is not currently in a game".to_owned())
                        .and_then(|game_executor_mtx| {
                            let mut game_executor = game_executor_mtx
                                .lock()
                                .map_err(|_| "Poisoned mutex".to_owned())?;
                            let timed_move =
                                game_executor.create_move(game_move.from, game_move.to)?;
                            game_executor.add_move(self.player.as_ref().unwrap(), timed_move)
                        }),
                    _ => Err("The provided message type was not found.".to_owned()),
                }
            }();
            match result {
                Ok(_) => (),
                Err(e) => {
                    let mut sink = self.sink.lock().unwrap();
                    let _ = sink.start_send(Message::from(
                        serde_json::to_string(&MessageType::Error(e)).unwrap(),
                    ));
                }
            };
        }
    }

    pub async fn handle_new_client(&mut self) {
        use ipg_core::protocol::messages::{GameList, MessageType};
        let mut sink = self.sink.lock().unwrap();
        let result = match self.instance.games.read() {
            Ok(mut games) => {
                let games_metadata = games
                    .iter()
                    .map(|(key, val)| GameMetadata {
                        game_id: key.clone(),
                        map_id: val.lock().unwrap().game.map.name.clone()
                    })
                    .collect();
                let seralized = serde_json::to_string(&MessageType::GameList(GameList {
                    games: games_metadata,
                }));
                let _ = sink.start_send(Message::from(seralized.unwrap()));
                let map_manager = self.instance.map_manager.lock().expect("Game state corrupted by poisned mutex");
                let message = &MessageType::MapList(map_manager.maps() );
                let seralized = serde_json::to_string(message);
                let _ = sink.start_send(Message::from(seralized.unwrap()));
                Ok(())
            }
            Err(_) => Err("Game state corrupted by poisned mutex.".to_owned()),
        };
        if let Err(err_msg) = result {
            let _ = sink.start_send(Message::from(err_msg));
        }
    }
}
