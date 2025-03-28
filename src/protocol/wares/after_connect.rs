use crate::router::State;
use crate::logging::decrement_total_connected;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;

#[async_trait]
pub trait AfterConnect: Debug + Send + Sync {
    async fn execute(&self, state: Arc<Mutex<State>>);
}

#[derive(Debug)]
pub struct DefaultAfterConnect;

#[async_trait]
impl AfterConnect for DefaultAfterConnect  {
    async fn execute(&self, _: Arc<Mutex<State>>) {
        decrement_total_connected();
    }
}