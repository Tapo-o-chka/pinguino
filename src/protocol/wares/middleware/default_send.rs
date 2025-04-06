use async_trait::async_trait;
use crate::{protocol::{wares::Middleware, request::{Request, Version}, response::{Response, ResponseCode, ResponseBuilder}}, router::State};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;

/// ## `DefaultMiddleware`
/// 
/// This is the default implementator of the [`Middleware`] trait for `Send` [`Method`].
/// 
/// ## How does it look in the human way:
/// ```
/// #[async_trait]
/// impl Middleware for DefaultMiddleware {
///     async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response> {
///         let state = state.lock().await;
///         if let Some(name) = state.varmap.get::<String>() {
///             let dt = Utc::now();
///             let naive_utc = dt.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
///             
///             let response = ResponseBuilder::new()
///                 .version(Version::CHAT10)
///                 .code(ResponseCode::OK)
///                 .custom_init()
///                 .custom_insert("Time".to_string(), naive_utc)
///                 .user(name.clone())
///                 .message(req.value)
///                 .build();
///             
///             match response {
///                 Ok(val) => {
///                     return Ok(val);
///                 },
///                 Err(_) => {
///                     let response = ResponseBuilder::new()
///                         .version(Version::CHAT10)
///                         .code(ResponseCode::ParseError)
///                         .build()
///                         .unwrap();
/// 
///                     return Err(response);
///                 }
///             }
///         }
/// 
///         return Err(ResponseBuilder::new()
///             .version(Version::CHAT10)
///             .code(ResponseCode::InvalidName)
///             .build()
///             .unwrap());
///     }
/// }
/// ```
/// 
/// [`Method`]: crate::protocol::request::Method
#[derive(Debug)]
pub struct DefaultMiddleware;

#[async_trait]
impl Middleware for DefaultMiddleware {
    async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response> {
        let state = state.lock().await;
        if let Some(name) = state.varmap.get::<String>() {
            let dt = Utc::now();
            let naive_utc = dt.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
            
            let response = ResponseBuilder::new()
                .version(Version::CHAT10)
                .code(ResponseCode::OK)
                .custom_init()
                .custom_insert("Time".to_string(), naive_utc)
                .user(name.clone())
                .message(req.value)
                .build();
            
            match response {
                Ok(val) => {
                    return Ok(val);
                },
                Err(_) => {
                    let response = ResponseBuilder::new()
                        .version(Version::CHAT10)
                        .code(ResponseCode::ParseError)
                        .build()
                        .unwrap();

                    return Err(response);
                }
            }
        }

        return Err(ResponseBuilder::new()
            .version(Version::CHAT10)
            .code(ResponseCode::InvalidName)
            .build()
            .unwrap());
    }
}