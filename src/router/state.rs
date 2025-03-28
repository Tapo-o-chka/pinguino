use tokio::sync::Mutex;
use std::sync::Arc;
use crate::protocol::{varmap::Varmap, wares::AfterConnect};
use super::App;

/// `State` is the struct that repesenets application state. Dont confuse your self with App.
/// `App` is the state of the client. `State` is the state of the whole application
/// 
#[derive(Debug, Clone)]
pub struct State {
    pub app: Arc<Mutex<App>>,                       // Linking app state
    pub after: Arc<Option<Box<dyn AfterConnect>>>,  // Saving After function to not forget it
    pub varmap: Varmap,                             // Connection-long Varmap
}

impl State {
    pub fn new(app: Arc<Mutex<App>>, after: Arc<Option<Box<dyn AfterConnect>>>) -> Self {
        State {
            app,
            after,
            varmap: Varmap::new(),
        }
    }
}