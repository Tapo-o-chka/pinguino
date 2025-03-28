use async_trait::async_trait;
use crate::{protocol::{wares::Middleware, request::{Request, Version}, response::{Response, ResponseCode, ResponseBuilder}}, router::State};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct DefaultMiddleware;

#[async_trait]
impl Middleware for DefaultMiddleware {
    async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response> {
        let mut state = state.lock().await;
        let token = &req.value;
        if let Some(name) = state.clone().app.lock().await.auth.get(token) { // Clonning whole state.custom, locking, awaiting is crazy, need to change that for sure
            state.varmap.insert(name.clone());

            return Ok(ResponseBuilder::new().version(Version::CHAT10).code(ResponseCode::AuthOK).build().unwrap());
        } 

        Err(ResponseBuilder::new().version(Version::CHAT10).code(ResponseCode::Unauthorized).build().unwrap())
    }
}