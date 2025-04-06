use std::str::FromStr;
use std::fmt::Debug;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use crate::client::{ClientError, ClientState};
use crate::protocol::request::Request;

#[async_trait::async_trait]
pub trait SendTrait: Debug + Send + Sync {
    async fn send(&self, state: Arc<Mutex<ClientState>>, message: String) -> Result<(), ClientError>;
}

#[derive(Debug)]
pub struct DefaultSend;

#[async_trait::async_trait]
impl SendTrait for DefaultSend {
    async fn send(&self, state: Arc<Mutex<ClientState>>, message: String) -> Result<(), ClientError> {
        println!("Im here!");
        let message = message.trim_end_matches('\n');
        let addr = SocketAddr::from_str("127.0.0.1:9999").unwrap(); // Just a place holder.
        let request = match Request::parse(&format!("<CHAT \\ 1.0>\n<Method@Send>\n<Message@'{0}'>", message), Arc::new(addr)) {
            Ok(val) => val,
            Err(e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [SEND] Failed to parse the request with error {:?}", e);
                
                return Err(ClientError::ParseError(e));
            }
        };
        
        let locked = state.lock().await;
        match locked.out_sender.send(request).await {
            Ok(_) => Ok(()),
            Err(_e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [SEND] Failed to send the request with error {_e}");

                Err(ClientError::InternalError)
            }
        }
    }
}