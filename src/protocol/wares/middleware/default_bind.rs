use async_trait::async_trait;
use crate::{protocol::{wares::Middleware, request::{Request, Version}, response::{Response, ResponseCode, ResponseBuilder}}, router::State};
use std::sync::Arc;
use tokio::sync::Mutex;

/// ## `DefaultMiddleware`
/// 
/// This is the default implementator for the [`Middleware`] for the `Bind` [`Method`]
/// 
/// [`Method`]: crate::protocol::request::Method
/// 
/// ## How does it look in the human way
/// 
/// ```
/// #[async_trait]
/// impl Middleware for DefaultMiddleware {
///     async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response> {
///         let state = state.lock().await;
///         let name = &req.value;
/// 
///         // Idk why it wanst me 
///         let _ = match state.app.lock().await.register(name.clone()) { // Crazy amount of things need to be done, todo! change it 
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
///     }
/// }
/// ```
#[derive(Debug)]
pub struct DefaultMiddleware;

#[async_trait]
impl Middleware for DefaultMiddleware {
    async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response> {
        let state = state.lock().await;
        let name = &req.value;

        // Idk why it wanst me 
        let _ = match state.app.lock().await.register(name.clone()) { // Crazy amount of things need to be done, todo! change it 
            Ok(token) => {
                return Ok(ResponseBuilder::new()
                    .version(Version::CHAT10)
                    .code(ResponseCode::AuthOK)
                    .token(token)
                    .build()
                    .unwrap()
                );
            },
            Err(_) => {
                return Err(ResponseBuilder::new()
                    .version(Version::CHAT10)
                    .code(ResponseCode::AlreadyTaken)
                    .build()
                    .unwrap()
                );
            }
        };
    }
}