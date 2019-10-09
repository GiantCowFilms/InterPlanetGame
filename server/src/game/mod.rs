use std::future::Future;
use ipg_core::game::Game;

pub struct AsyncGameExecutor {
    pub game: Game,
    on_move: Vec<Box<Future<Output = ()> + Send + Sync>>,
    on_state: Vec<Box<Future<Output = ()> + Send + Sync>>
}

impl AsyncGameExecutor {
    pub fn new(game: Game) -> AsyncGameExecutor {
        AsyncGameExecutor {
            game,
            on_move: Vec::new(),
            on_state: Vec::new()
        }
    }

    pub fn on_move<F>(&mut self,handler: F) 
        where F: Future<Output = ()> + Send + Sync + 'static,
     {
        self.on_move.push(Box::new(handler));
    }

    pub fn on_state<F>(&mut self,handler: F) 
        where F: Future<Output = ()> + Send + Sync + 'static,
     {
        self.on_state.push(Box::new(handler));
    }
}