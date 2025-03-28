use crate::router::State;
use crate::logging::increment_total_connected;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;

#[async_trait]
pub trait BeforeConnect: Debug + Send + Sync {
    async fn execute(&self, state: Arc<Mutex<State>>);
}

#[derive(Debug)]
pub struct DefaultBeforeConnect;

#[async_trait]
impl BeforeConnect for DefaultBeforeConnect  {
    async fn execute(&self, _: Arc<Mutex<State>>) {
        increment_total_connected();
    }
}