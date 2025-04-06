use std::sync::Arc;
use std::fmt::Debug;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use crate::client::ClientState;
use crate::protocol::response::Response;

#[async_trait::async_trait]
pub trait SubscribeTrait: Debug + Send + Sync {
    async fn subscribe(&self, state: Arc<Mutex<ClientState>>) -> Arc<Mutex<UnboundedReceiver<Response>>>;
}

#[derive(Debug)]
pub struct DefaultSubscribe;

#[async_trait::async_trait]
impl SubscribeTrait for DefaultSubscribe {
    async fn subscribe(&self, state: Arc<Mutex<ClientState>>) -> Arc<Mutex<UnboundedReceiver<Response>>> {
        let locked = state.lock().await;
        locked.in_reciever.clone()
    }
}