use crate::{logging::Message, protocol::wares::{EndingBytesware, Middleware, StartingBytesware}, router::State};
use crate::protocol::request::{RawRequest, Request, Version, ParseError};
use crate::protocol::response::{Response, ResponseBuilder, ResponseCode};
use chrono::Utc;
use tokio::{sync::{mpsc::Sender, Mutex}, task::Id};
use tracing::warn;
use std::sync::Arc;
use async_trait::async_trait;
use std::time::Instant;

#[derive(Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

#[allow(dead_code)]
impl Color {
    pub fn to_string(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

#[derive(Debug)]
pub struct SendStartingBytesware;
#[derive(Debug)]
pub struct SendEndingBytesware;
#[derive(Debug)]
pub struct SendMiddleware;

#[async_trait]
impl StartingBytesware for SendStartingBytesware {
    async fn bytesware(&self, mut raw_req: RawRequest) -> Result<Request, Response> {
        let color_res = fill_bytes(&mut raw_req.bytes);
        let request = Request::from_raw_request(raw_req);
    
        match request {
            Ok(mut req) => {
                if let Ok(color) =  color_res {
                    println!("there is color!");
                    req.varmap.insert(color);
                }
                let cur = Instant::now();
                req.varmap.insert(cur);
                return Ok(req);
            },
            Err(e) => {
                let response = ResponseBuilder::new()
                    .version(Version::CHAT10);

                let code = match e {
                    ParseError::InvalidFormat => ResponseCode::ParseError,
                    ParseError::InvalidKey => ResponseCode::Unauthorized,
                    ParseError::MissingMethod => ResponseCode::InvalidHeader,
                    ParseError::MissingRequestValue => ResponseCode::InvalidName,
                    ParseError::MissingVersion => ResponseCode::InvalidHeader,
                    ParseError::MissingCode => ResponseCode::Error, // How in the world would you get it here?
                    ParseError::NotFound => ResponseCode::Error,
                };

                warn!(
                    message = "Parse error"
                );

                return Err(response.code(code).build().unwrap());
            }
        }
    }
}

#[async_trait]
impl Middleware for SendMiddleware {
    async fn middleware(&self, req: Request, state: Arc<Mutex<State>>) -> Result<Response, Response> {
        let state = state.lock().await;
        
        if let Some(name) = state.varmap.get::<String>() {
            let dt = Utc::now();
            let naive_utc = dt.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
            
            let val = req.varmap.get::<Instant>().unwrap().elapsed().as_micros();
            let app = state.app.lock().await;
            let message = app.extension.get::<&str>();
            println!("<Elapsed: {:?} {:?}>", req.varmap.get::<Instant>().unwrap().elapsed(), message);
            //let app = state.app.lock().await;
            if let Some(rx) = app.extension.get::<Sender<Message>>() {
                if let Some(thread_id) = state.varmap.get::<Id>() {
                    let message = Message::Elapsed(*thread_id, Utc::now(), val);
    
                    match rx.send(message).await {
                        Ok(_) => {
                            #[cfg(feature = "debug_full")]
                            println!("--> [SUB:{thread_id}] Sent warn logs ");
                        },
                        Err(_) => {
                            #[cfg(feature = "debug_light")]
                            println!("<<< [SUB:{thread_id}] Failed failed FAILED");
                        }
                    }
                }
            }
            
            tracing::info!(
                value = %val
            );
            if let Some(color) = req.varmap.get::<Color>() { 
                // Was just lazy to split it up in two parts, and just insert in the middle, sorry
                return Ok(ResponseBuilder::new()
                    .version(Version::CHAT10)
                    .code(ResponseCode::OK)
                    .custom_init()
                    .custom_insert("Time".to_string(), naive_utc)
                    .varmap_insert(color.clone())
                    .user(name.clone())
                    .message(req.value)
                    .build()
                    .unwrap()
                );
            }

            return Ok(ResponseBuilder::new()
                .version(Version::CHAT10)
                .code(ResponseCode::OK)
                .custom_init()
                .custom_insert("Time".to_string(), naive_utc)
                .user(name.clone())
                .message(req.value)
                .build()
                .unwrap()
            );
        }

        warn!(
            message = "Invalid name"
        );
        return Err(ResponseBuilder::new()
            .version(Version::CHAT10)
            .code(ResponseCode::InvalidName)
            .build()
            .unwrap());
    }
}

#[async_trait]
impl EndingBytesware for SendEndingBytesware {
    async fn bytesware(&self, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]> {
        match res {
            Ok(res) => {
                let mut response_bytes = match res.as_bytes() {
                    Ok(val) => val,
                    Err(_) => {
                        warn!(
                            message = "Failed to make generate bytes"
                        );

                        return Err(ResponseBuilder::new()
                            .version(Version::CHAT10)
                            .code(ResponseCode::FatalError)
                            .build()
                            .unwrap()
                            .as_bytes()
                            .unwrap()
                        );
                    }
                };

                if let Some(varmap) = res.varmap {
                    if let Some(color) = varmap.get::<Color>() {
                        let symb = '#' as u8;
                        response_bytes[512 - 1] = symb;
                        response_bytes[512 - 2] = color.blue;
                        response_bytes[512 - 3] = color.green;
                        response_bytes[512 - 4] = color.red; 
                        response_bytes[512 - 5] = symb;
                    }
                }

                return Ok(response_bytes)
            },
            Err(res) => {
                return Err(res.as_bytes().unwrap());
            }
        }
    }
}

fn fill_bytes(bytes: &mut [u8; 512]) -> Result<Color, ()> {
    let symb = '#' as u8;
    if !(bytes[512 - 1] == bytes[512 - 5] && bytes[512 - 1] == symb) {
        return Err(())
    }
    // Step 2: Extract hexnumbers
    let blue = bytes[512 - 2];
    let green = bytes[512 - 3];
    let red = bytes[512 - 4];

    let color = Color { red, green, blue };
    // Step 3: Dont forget to 0 retrieved bytes
    bytes[512-5..=512-1].fill(0);
    // Step 4: Done! Return color.
    Ok(color)
}