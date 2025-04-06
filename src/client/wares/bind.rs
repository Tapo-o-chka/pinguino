use std::sync::Arc;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::Mutex};
use std::fmt::Debug;

use crate::{client::{ClientError, ClientState}, protocol::response::Response};

/// ## `Bind`
/// 
/// This trait is responsible for implementing `Bind` request from client to the server.
/// 
/// ## Ideal usage
/// 1. Use `bind()` for binding name
/// 2. Use `bindt()` for binding token to the current state
/// 
/// ## How it looks like in the human way
/// 
/// ```
/// #[async_trait::async_trait]
/// pub trait BindTrait: Debug + Send + Sync {
///     async fn bind(&self, state: Arc<Mutex<ClientState>>, name: String) -> Result<(), ClientError>;
/// 
///     async fn bindt(&self, state: Arc<Mutex<ClientState>>, token: String);
/// }
/// ```
/// 
/// ## How to implement
/// See [`DefaultBind`] for default implementation code.
#[async_trait::async_trait]
pub trait BindTrait: Debug + Send + Sync {
    async fn bind(&self, state: Arc<Mutex<ClientState>>, name: String) -> Result<(), ClientError>;

    async fn bindt(&self, state: Arc<Mutex<ClientState>>, token: String);
}

/// ## `DefaultBind`
/// 
/// This is the default implementator of the `Bind`. It consist of general functionality
/// needed to send bind request - Taking name -> returning token extracted from the server response.
/// Optionally, if you waant to be robust, you could create custom implementation with storing
/// everything in the `client state` or you could save `token` in the secured storage in sidie of 
/// your custom implementation.
/// 
/// ## How does it look in the human way:
/// 
/// ```
/// #[derive(Debug)]
/// pub struct DefaultBind;
/// 
/// impl BindTrait for DefaultBind {
///     async fn bind(state: Arc<Mutex<ClientState>>, name: String) -> Result<(), ClientError> {
///         let locked = state.lock().await;
///         let mut stream = match TcpStream::connect(locked.target).await {
///             Ok(val) => val,
///             Err(e) => { return Err(ClientError::CouldntConnect(e)); }
///         };
/// 
///         match stream.write(format!("<CHAT \\ 1.0>\n<Method@Bind>\n<Name@{name}>").as_bytes()).await {
///             Ok(0) => {
///                 #[cfg(feature = "debug_light")]
///                 println!("<<< [BIND] Closed connection, before it needed");
/// 
///                 return Err(ClientError::ClosedConnection);
///             },
///             Ok(_val) => {
///                 #[cfg(feature = "debug_full")]
///                 println!("--> [BIND] Sent {_val} bytes to the server");
/// 
///                 let mut read_buf = [0u8; 512];
///                 match stream.read(&mut read_buf).await {
///                     Ok(0) => {
///                         #[cfg(feature = "debug_light")]
///                         println!("<<< [BIND] Closed connection, before it needed");
/// 
///                         return Err(ClientError::ClosedConnection);                        
///                     },
///                     Ok(_val) => {
///                         #[cfg(feature = "debug_full")]
///                         println!("--> [BIND] Read {_val} bytes to the server");
/// 
///                         let response = match Response::from_bytes(&read_buf) {
///                             Ok(val) => val,
///                             Err(e) => { 
///                                 #[cfg(feature = "debug_light")]
///                                 println!("<<< [BIND] Failed to parse response with error {:?}", e);
/// 
///                                 return Err(ClientError::ParseError(e)); },
///                         };
///                         if let Some(token) = response.token {
///                             return Ok(token)
///                         }
///                         // No real need for else, right?
///                         return Err(ClientError::MissingToken);
///                     },
///                     Err(e) => {
///                         #[cfg(feature = "debug_light")]
///                         println!("<<< [BIND] Failed to read from the server with error {e}");
/// 
///                         return Err(ClientError::ReadingFailed(e));
///                     }
///                 }
///             },
///             Err(e) => {
///                 #[cfg(feature = "debug_light")]
///                 println!("<<< [BIND] Failed to send request to the server with error: {e}");
/// 
///                 return Err(ClientError::SendingFailed(e));
///             }
///         }
///     }
/// 
///     async fn bindt(&self, state: Arc<Mutex<ClientState>>, token: String) {
///         let mut locked = state.lock().await;
///         locked.token = Some(token);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct DefaultBind;

#[async_trait::async_trait]
impl BindTrait for DefaultBind {
    async fn bind(&self, state: Arc<Mutex<ClientState>>, name: String) -> Result<(), ClientError> {
        // locking state to recieve target
        let mut locked = state.lock().await;

        // Connecting to the server
        let mut stream = match TcpStream::connect(locked.target).await {
            Ok(val) => val,
            Err(e) => { return Err(ClientError::CouldntConnect(e)); }
        };

        // Writing to the stream
        match stream.write(format!("<CHAT \\ 1.0>\n<Method@Bind>\n<Name@{name}>").as_bytes()).await {
            Ok(0) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [BIND] Closed connection, before it needed");

                return Err(ClientError::ClosedConnection);
            },
            Ok(_val) => {
                #[cfg(feature = "debug_full")]
                println!("--> [BIND] Sent {_val} bytes to the server");

                // Server responded -> We need to extract `token`
                let mut read_buf = [0u8; 512];
                match stream.read(&mut read_buf).await {
                    Ok(0) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [BIND] Closed connection, before it needed");

                        // Unable to extract token, when connection is closed.
                        return Err(ClientError::ClosedConnection);                        
                    },
                    Ok(_val) => {
                        #[cfg(feature = "debug_full")]
                        println!("--> [BIND] Read {_val} bytes from the server");

                        // Response extraction
                        let response = match Response::from_bytes(&read_buf) {
                            Ok(val) => val,
                            Err(e) => { 
                                #[cfg(feature = "debug_light")]
                                println!("<<< [BIND] Failed to parse response with error {:?}", e);

                                return Err(ClientError::ParseError(e)); },
                        };

                        // Trying to get the token
                        if let Some(token) = response.token {
                            locked.token = Some(token);

                            return Ok(())
                        }

                        return Err(ClientError::MissingToken);
                    },
                    Err(e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [BIND] Failed to read from the server with error {e}");

                        return Err(ClientError::ReadingFailed(e));
                    }
                }
            },
            Err(e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [BIND] Failed to send request to the server with error: {e}");

                return Err(ClientError::SendingFailed(e));
            }
        }
    }

    async fn bindt(&self, state: Arc<Mutex<ClientState>>, token: String) {
        let mut locked = state.lock().await;
        locked.token = Some(token);
    }
}