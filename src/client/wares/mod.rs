//! ## `Client wares`
//! 
//! This module holds main `traits` and implementations for the `Client` object
//! It is highly inspired by [`Router`].
//! 
//! [`Router`]: crate::router::Router
pub mod bind;
pub mod handshake;
pub mod send;
pub mod subscribe;
pub mod terminate;

pub use bind::{BindTrait, DefaultBind};
pub use handshake::{HandshakeTrait, DefaultHandshake};
pub use send::{SendTrait, DefaultSend};
