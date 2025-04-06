//! ## `BeforeConnect`
//! 
//! Well, it is module. `BeforeConnect` is executed, before connection is ended.
//! 
//! Go look at [`BeforeConnect`] trait or [`DefaultBeforeConnect`] struct.
use crate::router::State;
use super::increment_total_connected;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;

/// ## `BeforeConnect`
/// I m asuming that it looks like an abomination to you, but i hope following will clear it out.
/// 
/// This trait is for executing some code on connection start, so before even [`StartingBytesware`]
/// is executed. You could insert things inside of the `state` using its `varmap` field, or you could
/// increment global static variable.
/// 
/// ## How it look in human way:
/// ```
/// #[async_trait]
/// pub trait BeforeConnect: Debug + Send + Sync {
///     async fn execute(&self, state: Arc<Mutex<State>>);
/// }
/// ```
/// [`StartingBytesware`]: crate::protocol::wares::StartingBytesware
#[async_trait]
pub trait BeforeConnect: Debug + Send + Sync {
    async fn execute(&self, state: Arc<Mutex<State>>);
}

/// ## `DefaultBeforeConnection`
/// 
/// This struct is the defualt implementator of the [`BeforeConnect`] trait. It is used to act
/// as is inside of the [`Router`] by default. If you want to disable it in the [`Router`] you
/// should manually set `before` field as `None`.
/// 
/// ## Example
/// This example is to show you, how implmentation should look like:
/// ```
/// #[async_trait]
/// impl BeforeConnect for DefaultBeforeConnect  {
///     async fn execute(&self, _: Arc<Mutex<State>>) {
///         increment_total_connected();
///     }
/// }
/// ```
/// 
/// *Dont worry about [`increment_total_connected`], its just default function, that aint doing much*
/// 
/// [`Router`]: crate::router::Router
#[derive(Debug)]
pub struct DefaultBeforeConnect;

#[async_trait]
impl BeforeConnect for DefaultBeforeConnect  {
    async fn execute(&self, _: Arc<Mutex<State>>) {
        increment_total_connected();
    }
}