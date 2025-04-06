//! ## `AfterConnect`
//! 
//! Well, it is module. `AfterConnect` is executed, after connection is ended.
//! 
//! Go look at [`AfterConnect`] trait or [`DefaultAfterConnect`] struct.
use crate::router::State;
use super::decrement_total_connected;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;

/// ## `AfterConnect`
/// 
/// Well, im asuming that it looks like an abomination to you, but please, be pacient.
/// 
/// This trait is for executing piece of code after the connection is ended. You could retrieve collected info,
/// from `varmap`, if you putted anything inside of it, or just use some global static variable as atomic
/// incrementor.
/// 
/// ## How it look in human way:
/// ```
/// #[async_trait]
/// pub trait AfterConnect: Debug + Send + Sync {
///     async fn execute(&self, state: Arc<Mutex<State>>);
/// }
/// ```
/// Much better, right?
/// 
/// *For implementation example, look at [`DefaultAfterConnect`]*
#[async_trait]
pub trait AfterConnect: Debug + Send + Sync {
    async fn execute(&self, state: Arc<Mutex<State>>);
}

/// ## `DefaultAfterConnection`
/// 
/// This struct is the default implementator of the [`AfterConnect`] trait. It is used
/// to act as is inside of the [`Router`] by default. If you want to disable it in the [`Router`],
/// you should set it to `None` manually
/// 
/// [`Router`]: crate::router::Router
/// 
/// ## Example
/// This example is to show you, how implementation should look like:
/// ```
/// #[async_trait]
/// impl AfterConnect for DefaultAfterConnect  {
///     async fn execute(&self, _: Arc<Mutex<State>>) {
///         decrement_total_connected();
///     }
/// }
/// ```
/// *Dont worry about [`decrement_total_connected`], its just default function, that aint doing much*
#[derive(Debug)]
pub struct DefaultAfterConnect;

#[async_trait]
impl AfterConnect for DefaultAfterConnect  {
    async fn execute(&self, _: Arc<Mutex<State>>) {
        decrement_total_connected();
    }
}