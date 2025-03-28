use async_trait::async_trait;
use std::fmt::Debug;

use crate::protocol::response::Response;

pub mod default_bind;
pub mod default_handshake;
pub mod default_send;

#[async_trait]
pub trait EndingBytesware: Debug + Send + Sync {
    async fn bytesware(&self, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]>;
}
