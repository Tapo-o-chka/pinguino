use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;

use crate::protocol::{response::Response, request::Request};
use crate::router::State;

pub mod default_bind;
pub mod default_handshake;
pub mod default_send;

#[async_trait]
pub trait Middleware: Debug + Send + Sync {
    async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response>;
}