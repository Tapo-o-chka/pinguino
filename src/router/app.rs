use std::collections::HashMap;
use uuid::Uuid;
use crate::protocol::Varmap;

/// This things is subfield of the [`State`]. But  `App` is actually app state, and [`State`] is connection state.
/// I dont know on what i was while writing it.
/// The goal of `App` struct is to be the tool, that is used by developer inside of mostly [`Middleware`],
/// but could be used for `bytesware`'s also.
/// 
/// Why cant i seperate `App` and [`State`] to not lock() two times? No idea. Will figure it out on later patches. 
/// 
/// ## Additional info
/// *`self.extension` field is clone of the Router varmap, in which we were inserting values at the router creation*
/// If im not mistaking LOL.
/// 
/// ## Example
/// 
/// ```
/// 
/// // Lets say that we inserted some value 
/// {
///     let router = RouterBuilder::new()
///         .insert("Cool message")
///         .build();
/// }
/// 
/// impl Middleware for DefaultMiddleware {
///     async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response> {
///         let state = state.lock().await;
///         let name = &req.value;
///
///         // This is example for Bind middleware needs
///         let _ = match state.app.lock().await.register(name.clone()) {
///             Ok(token) => {
///                 return Ok(ResponseBuilder::new()
///                     .version(Version::CHAT10)
///                     .code(ResponseCode::AuthOK)
///                     .token(token)
///                     .build()
///                     .unwrap()
///                 );
///             },
///             Err(_) => {
///                 return Err(ResponseBuilder::new()
///                     .version(Version::CHAT10)
///                     .code(ResponseCode::AlreadyTaken)
///                     .build()
///                     .unwrap()
///                 );
///             }
///         };
/// 
///         if let Some(message) = state.app.lock().await.get::<&str>() {
///             println!("There was the message! {:?}", message);
///         }
///     }
/// }
/// ```
/// 
/// [`State`]: crate::router::State
/// [`Middleware`]: crate::protocol::wares::middleware
#[derive(Debug, Clone)]
pub struct App {
    pub auth: HashMap<String, String>,
    pub names: HashMap<String, String>,
    pub extension: Varmap,
}

impl App {
    pub fn new(extension: Varmap) -> Self {
        App {
            auth: HashMap::new(),
            names: HashMap::new(),
            extension
        }
    }

    /// This function is intended to be used inside of `Bind` `middleware`.
    pub fn register(&mut self, name: String) -> Result<String, ()> {
        if self.names.get(&name) == None {
            let token = Uuid::new_v4().to_string();
            self.auth.insert(token.clone(), name.clone());
            self.names.insert(name, token.clone());
            return Ok(token);
        }
        Err(())
    } 
}