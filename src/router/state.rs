use tokio::sync::Mutex;
use std::sync::Arc;
use crate::protocol::{Varmap, wares::AfterConnect};
use super::App;

/// `State` is the struct that repesenets application state. Dont confuse your self with App.
/// `App` is the state of the client. `State` is the state of the whole application
/// 
/// `State` varmap holds values, that are valid connection-long
/// 
/// ## Examples
/// 
/// ### Example of usage
/// 
/// ```
/// // Somewhere inside of the StartingBytesware
/// async fn bytesware(&self, state: Arc<Mutex<State>>, raw_req: RawRequest) -> Result<Request, Response> {
///     let locked_state = state.lock().await; 
///     
///     // For example we inserted timestamp to know how long it takes to process request
///     locked_state.insert(Instant::now());
///     
///     todo!()
/// }
/// 
/// // Somewhere inside of the EndingBytesware
/// async fn bytesware(&self, state: Arc<Mutex<State>>, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]> {
///     let locked_state = state.lock().await;
/// 
///     // Print final timer!
///     if let Some(timer) = locked_state.get::<Instant>() {
///         println!(">>> [SUB] Finished request in {0}", timer.elapsed().as_micros());
///     }
/// 
///     todo!()
/// }
/// ```
/// 
/// **Attention** `String` field is reserved for the `username`.
/// 
#[derive(Debug, Clone)]
pub struct State {
    pub app: Arc<Mutex<App>>,                       // Linking app state
    pub after: Arc<Option<Box<dyn AfterConnect>>>,  // Saving After function to not forget it
    pub varmap: Varmap,                             // Connection-long Varmap
}

impl State {
    /// Creates new `State` for the connection.
    /// ## Examples
    /// 
    /// ```
    /// // Lets say that we already have `Router` and `App`
    /// let state = Arc::new(Mutex::new(State::new(app.clone(), router.after.clone()))); 
    /// ```
    pub fn new(app: Arc<Mutex<App>>, after: Arc<Option<Box<dyn AfterConnect>>>) -> Self {
        State {
            app,
            after,
            varmap: Varmap::new(),
        }
    }
}