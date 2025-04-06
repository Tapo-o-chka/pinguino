//! ## `DefaultEndingBytesware` for [`Method`]::Handshake
//! 
//! click on the struct for more info
//! 
//! [`Method`]: crate::protocol::request::Method
use std::sync::Arc;
use async_trait::async_trait;
use crate::protocol::request::Version;
use crate::protocol::response::{Response, ResponseBuilder, ResponseCode};
use crate::protocol::wares::EndingBytesware;
use crate::router::State;
use tokio::sync::Mutex;

/// ## `DefaultEndingBytesware`
/// 
/// This is the default implementor for the [`EndingBytesware`] and [`Method`]::Handshake
/// 
/// ## How does it look like in the human
/// ```
/// #[async_trait]
/// impl EndingBytesware for DefaultEndingBytesware {
///     async fn bytesware(&self, _: Arc<Mutex<State>>, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]> {
///         match res {
///             Ok(res) => {
///                 match res.as_bytes() {
///                     Ok(val) => Ok(val),
///                     Err(_) => {
///                         let res = ResponseBuilder::new()
///                             .version(Version::CHAT10)
///                             .code(ResponseCode::ParseError)
///                             .build()
///                             .unwrap();
/// 
///                         Err(res.as_bytes().unwrap())
///                     }
///                 }
///             },
///             Err(res) => {
///                 match res.as_bytes() {
///                     Ok(val) => Err(val),
///                     Err(_) => {
///                         let res = ResponseBuilder::new()
///                             .version(Version::CHAT10)
///                             .code(ResponseCode::ParseError)
///                             .build()
///                             .unwrap();
/// 
///                         Err(res.as_bytes().unwrap())
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
/// [`Method`]: crate::protocol::request::Method
#[derive(Debug)]
pub struct DefaultEndingBytesware;

#[async_trait]
impl EndingBytesware for DefaultEndingBytesware {
    async fn bytesware(&self, _: Arc<Mutex<State>>, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]> {
        match res {
            Ok(res) => {
                match res.as_bytes() {
                    Ok(val) => Ok(val),
                    Err(_) => {
                        let res = ResponseBuilder::new()
                            .version(Version::CHAT10)
                            .code(ResponseCode::ParseError)
                            .build()
                            .unwrap();

                        Err(res.as_bytes().unwrap())
                    }
                }
            },
            Err(res) => {
                match res.as_bytes() {
                    Ok(val) => Err(val),
                    Err(_) => {
                        let res = ResponseBuilder::new()
                            .version(Version::CHAT10)
                            .code(ResponseCode::ParseError)
                            .build()
                            .unwrap();

                        Err(res.as_bytes().unwrap())
                    }
                }
            }
        }
    }
}