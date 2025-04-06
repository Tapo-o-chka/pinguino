//! ## `StartingBytesware`
//! 
//! This is the trait, which have `bytesware` function. It would be executed on request recieve
//! in the router. 
use async_trait::async_trait;
use crate::protocol::{request::{ParseError, RawRequest, Request, Version}, response::{Response, ResponseBuilder, ResponseCode}};
use crate::router::State;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;

/// ## `StartingByteswqare`
/// 
/// Well, im asuming that it looks like an abomination to you, but please, be pacient.
/// 
/// This function is designed to execute pieces of code on incoming requests. You could modify
/// `state`, or add sticky note via `varmap` field of the  generated `Request` / `Response`.
/// 
/// ## How it looks like in the human way
/// ```
/// #[async_trait]
/// pub trait StartingBytesware: Debug + Send + Sync {
///     async fn bytesware(&self, state: Arc<Mutex<State>>, raw_req: RawRequest) -> Result<Request, Response>;
/// }
///
/// ```
///
/// *For example on how to implement function, look at the [`DefaultStartingBytesware`]*
#[async_trait]
pub trait StartingBytesware: Debug + Send + Sync {
    async fn bytesware(&self, state: Arc<Mutex<State>>, raw_req: RawRequest) -> Result<Request, Response>;
}

/// ## `DefaultStartingBytesware`
/// 
/// This is the default implementator of the [`StartingBytesware`]. Its simple, and steady.
/// 
/// ## How it looks like in the human way
/// ```
/// #[async_trait]
/// impl StartingBytesware for DefaultStartingBytesware {
///     async fn bytesware(&self, _: Arc<Mutex<State>>, raw_req: RawRequest) -> Result<Request, Response> {
///         let request = Request::from_raw_request(raw_req);
///     
///         match request {
///             Ok(req) => {
///                 return Ok(req);
///             },
///             Err(e) => {
///                 let response = ResponseBuilder::new()
///                     .version(Version::CHAT10);
/// 
///                 // Should've moved it into some sort of function, but i want to give better experience
///                 let code = match e {
///                     ParseError::InvalidFormat => ResponseCode::ParseError,
///                     ParseError::InvalidKey => ResponseCode::Unauthorized,
///                     ParseError::MissingMethod => ResponseCode::InvalidHeader,
///                     ParseError::MissingRequestValue => ResponseCode::InvalidName,
///                     ParseError::MissingVersion => ResponseCode::InvalidHeader,
///                     ParseError::MissingCode => ResponseCode::Error, // How in the world would you get it here?
///                     ParseError::NotFound => ResponseCode::Error,
///                 };
/// 
///                 return Err(response.code(code).build().unwrap());
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct DefaultStartingBytesware;

#[async_trait]
impl StartingBytesware for DefaultStartingBytesware {
    async fn bytesware(&self, _: Arc<Mutex<State>>, raw_req: RawRequest) -> Result<Request, Response> {
        let request = Request::from_raw_request(raw_req);
    
        match request {
            Ok(req) => {
                return Ok(req);
            },
            Err(e) => {
                let response = ResponseBuilder::new()
                    .version(Version::CHAT10);

                // Should've moved it into some sort of function, but i want to give better experience
                let code = match e {
                    ParseError::InvalidFormat => ResponseCode::ParseError,
                    ParseError::InvalidKey => ResponseCode::Unauthorized,
                    ParseError::MissingMethod => ResponseCode::InvalidHeader,
                    ParseError::MissingRequestValue => ResponseCode::InvalidName,
                    ParseError::MissingVersion => ResponseCode::InvalidHeader,
                    ParseError::MissingCode => ResponseCode::Error, // How in the world would you get it here?
                    ParseError::NotFound => ResponseCode::Error,
                };

                return Err(response.code(code).build().unwrap());
            }
        }
    }
}