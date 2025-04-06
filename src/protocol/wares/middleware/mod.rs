//! ## `Middleware`
//! 
//! This is the module... go look at default implementators and [`Middleware`] trait.
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;

use crate::protocol::{response::Response, request::Request};
use crate::router::State;

pub mod default_bind;
pub mod default_handshake;
pub mod default_send;

/// ## `Middleware`
/// 
/// This is the trait, that holds function that is executed right after [`StartingBytesware`] depending on which [`Method`] is at [`Request`].
/// The implementator of the trait is held inside of the [`Router`].
///
/// ## How does it look like in the human way
/// ```
/// #[async_trait]
/// pub trait Middleware: Debug + Send + Sync {
///     async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response>;
/// }
/// ``` 
/// 
/// ## Specification (sort of what is expected to be done)
/// 
/// ### [`Method`]::Bind
/// Implementator for this [`Method`] in the [`Middleware`] execution function is expected to add `User`
/// header `value` to the state via `state.app.lock().await.register(name.clone())`, where `name` is the
/// `value`. Important to note, that it is not adding it to the client state, it adds to the whole app.
/// 
/// ### [`Method`]::Send
/// Implementator for this [`Method`] in the [`Middleware`] execution function is expected to form valid `message` (*not the message, but message
/// for the clients*) thawt would be sent to all listening clients.
/// 
/// ### [`Method`]::Handshake
/// Implementator for this [`Method`] in the [`Middleware`] execution function is expected to check if `token` is valid,
/// and if it is, add `name` to the `state.varmap::insert(name)`. It is retrievable by `state.varmap::get::<String>()`.
/// 
/// [`StartingBytesware`]: crate::protocol::wares::starting_bytesware
/// [`Method`]: crate::protocol::request::Method
/// [`Router`]: crate::router::Router
#[async_trait]
pub trait Middleware: Debug + Send + Sync {
    async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response>;
}