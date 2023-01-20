use crate::GameServer;
use futures::{stream, StreamExt};
use ipg_core::game::{Game, GameEvent, GameExecutor, Player};
use ipg_core::protocol::messages::{EnterGame, GameList, GameMetadata, MessageType};
use std::borrow::BorrowMut;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_tungstenite::tungstenite::Message;

use super::rejoin::generate_rejoin_code;

pub trait Captures<'a> {}

impl<'a, T> Captures<'a> for T {}

pub struct GameConnection {
    player: Option<Player>,
    current_game: Option<Arc<Mutex<GameExecutor>>>,
    sink: Sender<Message>,
    instance: Arc<GameServer>,
}

impl GameConnection {
    pub fn new(instance: Arc<GameServer>, sink: Sender<Message>) -> Self {
        GameConnection {
            player: None,
            current_game: None,
            sink: sink,
            instance,
        }
    }

    fn handle_game_event(mut sink: Sender<Message>, game: &mut Game, event: &GameEvent) {
        //let mut executor = executor.lock().unwrap();
        match event {
            GameEvent::Start => {
                let seralized = serde_json::to_string(&MessageType::StartGame).unwrap();
                let seralized2 = serde_json::to_string(&MessageType::Game(game.clone())).unwrap();
                tokio::spawn(async move {
                    let _ = sink.send(Message::from(seralized)).await;
                    let _ = sink.send(Message::from(seralized2)).await;
                });
            }
            GameEvent::Move(_game_move) => {
                // let seralized =
                //     serde_json::to_string(&MessageType::TimedGameMove(game_move.clone())).unwrap();
                // sink.start_send(Message::from(seralized));
                let seralized = serde_json::to_string(&MessageType::Game(game.clone())).unwrap();
                tokio::spawn(async move {
                    let _ = sink.send(Message::from(seralized)).await;
                });
            }
            GameEvent::PlayerLeave(_) | GameEvent::Player(_) => {
                let seralized =
                    serde_json::to_string(&MessageType::GamePlayers(game.players.clone())).unwrap();
                tokio::spawn(async move {
                    let _ = sink.send(Message::from(seralized)).await;
                });
            }
        }
    }

    pub fn handle_message<'a: 'c, 'b: 'c, 'c>(
        &'b mut self,
        message: &'a Message,
    ) -> impl Future<Output = ()> + Captures<'a> + Captures<'b> + 'c {
        async move {
            let result = self.handle_message_internal(message).await;
            match result {
                Ok(_) => (),
                Err(e) => {
                    let _ = self
                        .sink
                        .send(Message::from(
                            serde_json::to_string(&MessageType::Error(e)).unwrap(),
                        ))
                        .await;
                }
            }
        }
    }

    async fn handle_message_internal<'a: 'c, 'b: 'c, 'c>(
        &'b mut self,
        message: &'a Message,
    ) -> Result<(), String> {
        let message_body = message
            .to_text()
            .map_err(|_| "The recieved message could not be parsed as a string.".to_owned())?;
        let message_data = serde_json::from_str::<MessageType>(message_body)
            .map_err(|_| "Could not parse the provided message.".to_owned())?;
        //Inside message handlers, always lock sinks first to avoid deadlocks
        match message_data {
            MessageType::Ping => {
                let seralized = serde_json::to_string(&MessageType::Pong);
                let _ = self.sink.send(Message::from(seralized.unwrap())).await;
                Ok(())
            }
            MessageType::CreateGame(game_settings) => {
                //let mut sink = self.sink.lock().unwrap();
                let game = {
                    let maps = self.instance.map_manager.lock().await;
                    let map = maps.map_by_id(&game_settings.map_id).ok_or_else(|| {
                        format!("Map with id \"{}\" not found.", game_settings.map_id)
                    })?;
                    Game::new((*map).clone(), game_settings.config)
                };
                self.instance.add_game(game).await;
                Ok(())
            }
            MessageType::ExitGame => {
                let _ = self.sink.send(Message::from("ExitGame")).await;
                Ok(())
            }
            MessageType::EnterGame(EnterGame {
                game_id,
                rejoin_code,
            }) => {
                let mut games = self.instance.games.write().await;
                let game_executor_mtx = games.get_mut(&game_id).ok_or_else(|| {
                    format!("Could not find a game with an id of \"{}\"", &game_id)
                })?;
                let mut game_executor = game_executor_mtx.lock().await;
                let player = self
                    .player
                    .as_ref()
                    .ok_or_else(|| "Players must set a name before joining a game.".to_owned())?;
                let (game_player, rejoin_code) = if let Some(rejoin_code) = rejoin_code {
                    let rejoin_mtx = self.instance.rejoin_codes.lock().await;
                    let possession = rejoin_mtx
                        .get(&(game_id.clone() + &rejoin_code))
                        .ok_or_else(|| {
                            format!(
                                "Could not find a session with rejoin id of \"{}\"",
                                &rejoin_code
                            )
                        })?;
                    game_executor.remove_player(&Player {
                        possession: *possession,
                        ..player.clone()
                    });
                    (
                        Player {
                            possession: *possession,
                            ..player.clone()
                        },
                        rejoin_code,
                    )
                } else {
                    let rejoin_code = generate_rejoin_code();
                    (
                        Player {
                            possession: 0,
                            ..player.clone()
                        },
                        rejoin_code,
                    )
                };
                self.player = Some(
                    game_executor
                        .add_player(game_player)
                        .map_err(|_| "Too many players".to_owned())?,
                );
                let handler_sink = self.sink.clone();
                // Subscribe to game state
                game_executor.event_source.on_event(Box::new(
                    move |event: &GameEvent, game: &mut Game| {
                        GameConnection::handle_game_event(handler_sink.clone(), game, event);
                    },
                ));
                let seralized = serde_json::to_string(&MessageType::EnterGame(EnterGame {
                    game_id: game_id.clone(),
                    rejoin_code: Some(rejoin_code.clone()),
                }));
                let _ = self.sink.send(Message::from(seralized.unwrap())).await;
                if let Some(player) = &self.player {
                    let mut rejoin_mtx = self.instance.rejoin_codes.lock().await;
                    rejoin_mtx.insert(game_id.clone() + &rejoin_code, player.possession);
                    let seralized =
                        serde_json::to_string(&MessageType::Possession(player.possession as u32));
                    let _ = self.sink.send(Message::from(seralized.unwrap())).await;
                };
                if game_executor.game.state.is_some() {
                    // Send game state
                    let time = game_executor.get_time();
                    // Step to the latest time so the client can use it
                    // to figure out the offset between it and the
                    // server. This offset does not account for latency,
                    // which will need to be fixed.
                    game_executor.step_to(time);
                    let seralized =
                        serde_json::to_string(&MessageType::Game(game_executor.game.clone()));
                    let _ = self.sink.send(Message::from(seralized.unwrap())).await;
                } else {
                    // Otherwise just send the player list
                    let seralized = serde_json::to_string(&MessageType::GamePlayers(
                        game_executor.game.players.clone(),
                    ));
                    let _ = self.sink.send(Message::from(seralized.unwrap())).await;
                }

                self.current_game = Some(game_executor_mtx.clone());
                Ok(())
            }
            MessageType::SetName(name_data) => {
                //Replace player to avoid mutexes/refcells and such
                self.player = Some(if let Some(mut player) = self.player.clone() {
                    player.name = name_data.name;
                    player
                } else {
                    Player {
                        name: name_data.name,
                        possession: 0, //Garbage data
                    }
                });
                Ok(())
            }
            MessageType::StartGame => {
                match self
                    .current_game
                    .as_ref()
                    .ok_or_else(|| "Player is not currently in a game".to_owned())
                {
                    Ok(game_executor) => game_executor.lock().await.start_game(),
                    Err(e) => Err(e),
                }
            }
            MessageType::GameMove(game_move) => {
                let game_executor_mtx = self
                    .current_game
                    .as_ref()
                    .ok_or_else(|| "Player is not currently in a game".to_owned());
                match game_executor_mtx {
                    Ok(game_exec_mtx) => {
                        let mut game_executor = game_exec_mtx.lock().await;
                        let timed_move = game_executor.create_move(game_move.from, game_move.to)?;
                        game_executor.add_move(self.player.as_ref().unwrap(), timed_move)?;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            MessageType::Time(_time) => {
                // Allows the client to get the server's clock time so
                // they can compute an offset between the server +
                // latency and their own clock.
                use std::time::SystemTime;
                let time = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let _ = self.sink.send(Message::from(
                    serde_json::to_string(&MessageType::Time(time)).unwrap(),
                ));
                Ok(())
            }
            _ => Err("The provided message type was not found.".to_owned()),
        }
    }

    pub async fn handle_new_client(&mut self) {
        let games = self.instance.games.read().await;
        let map_manager = self.instance.map_manager.lock().await;
        let message = &MessageType::MapList(map_manager.maps());
        let seralized = serde_json::to_string(message);
        let _ = self.sink.send(Message::from(seralized.unwrap())).await;
        let games_metadata = stream::iter(games.iter())
            .then(async move |(key, val)| {
                let game_exec = val.lock().await;
                GameMetadata {
                    game_id: key.clone(),
                    config: game_exec.game.config.clone(),
                    map_id: game_exec.game.map.name.clone(),
                }
            })
            .collect()
            .await;
        let seralized = serde_json::to_string(&MessageType::GameList(GameList {
            games: games_metadata,
        }));
        let _ = self.sink.send(Message::from(seralized.unwrap())).await;
    }

    pub async fn handle_client_exit(&mut self) {
        if let Some(game_executor_mtx) = &self.current_game {
            let mut game_executor = game_executor_mtx.lock().await;
            if let Some(player) = &self.player {
                game_executor.remove_player(&player);
            }
            if game_executor.game.players.len() < 1 {
                self.instance.remove_game(&game_executor.game_id).await;
            }
        };
    }
}
