//! ## `EndingBytesware`
//! 
//! This is the module... go look at default implementators and [`EndingBytesware`] trait.
use async_trait::async_trait;
use tokio::sync::Mutex;
use std::{fmt::Debug, sync::Arc};

use crate::{protocol::response::Response, router::State};

pub mod default_bind;
pub mod default_handshake;
pub mod default_send;

/// ## `EndingBytesware`
/// 
/// This is the trait, that holds function that is executed right after [`Middleware`] depending on which [`Method`] is at [`Request`].
/// The implementator of the trait is held inside of the [`Router`].
///
/// ## How does it look like in the human way
/// ```
/// #[async_trait]
/// pub trait EndingBytesware: Debug + Send + Sync {
///     async fn bytesware(&self, state: Arc<Mutex<State>>, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]>;
/// }
/// ``` 
/// 
/// [`Middleware`]: crate::protocol::wares::middleware
/// [`Request`]: crate::protocol::request::Request
/// [`Method`]: crate::protocol::request::Method
/// [`Router`]: crate::router::Router
#[async_trait]
pub trait EndingBytesware: Debug + Send + Sync {
    async fn bytesware(&self, state: Arc<Mutex<State>>, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]>;
}
