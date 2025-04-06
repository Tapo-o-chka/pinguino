use std::sync::Arc;
use std::fmt::Debug;
use tokio::sync::Mutex;
use crate::client::{ClientError, ClientState};


#[async_trait::async_trait]
pub trait TerminateTrait: Debug + Send + Sync {
    async fn terminate(&self, state: Arc<Mutex<ClientState>>) -> Result<(), ClientError>;
}

#[derive(Debug)]
pub struct DefaultTerminate;

#[async_trait::async_trait]
impl TerminateTrait for DefaultTerminate {
    async fn terminate(&self, state: Arc<Mutex<ClientState>>) -> Result<(), ClientError> {
        let mut locked = state.lock().await;
        if let Some(handle) = &locked.handle {
            if handle.is_finished() {
                #[cfg(feature = "debug_light")]
                println!("<<< [TERM] Failed to terminate the handle, because its already finished");
                
                locked.handle = None;
                return Err(ClientError::AlreadyFinished);
            }

            locked.handle = None;
            return Ok(());
        }
        #[cfg(feature = "debug_light")]
        println!("<<< [TERM] No handle is currently running");

        Err(ClientError::NoActiveHandle)
    } 
}