use crate::logging::{ErrorType, Message};
use crate::protocol::request::{RawRequest, Version};
use crate::protocol::response::ResponseCode;
use chrono::Utc;
use tokio::select;
use tokio::sync::Mutex;
use tokio::task::Id;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio::sync::{mpsc::{UnboundedSender as MpscSender, Sender}, broadcast::Receiver as BrReceiver};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::protocol::request::Method;
use crate::protocol::response::ResponseBuilder;

use super::{Routes, State};

pub async fn handle_send(mut stream: TcpStream, routes: Arc<Routes>, addr: Arc<SocketAddr>, state: Arc<Mutex<State>>, mut br_tx_sub: BrReceiver<[u8; 512]>, mp_tx_sub: MpscSender<[u8; 512]>, thread_id: Id) {
    let mut buf = [0u8; 512];
    loop {
        let readable = stream.read(&mut buf);
        let listener = br_tx_sub.recv();
        select! {
            val = readable => {
                match val {
                    Ok(0) => {
                        #[cfg(feature = "debug_light")]
                        println!(">>> [SUB:{thread_id}] Connection closed");

                        return;
                    },
                    Ok(_val) => {
                        #[cfg(feature = "debug_full")]
                        println!("--> [SUB:{thread_id}] Read {_val} bytes from the user");

                        let raw_req = RawRequest {
                            bytes: buf,
                            addr: addr.clone(),
                        };
                        
                        let req_res = routes.starting_bytesware.bytesware(raw_req).await;    

                        let req = match req_res {
                            Ok(val) => Ok(val),
                            Err(res) => {
                                #[cfg(feature = "debug_light")]
                                println!("<<< [SUB:{thread_id}] Failed to get parse request via bytesware");
                                
                                match res.as_bytes() {
                                    Ok(val) => {
                                        Err(val)
                                    },
                                    Err(_) => {
                                        #[cfg(feature = "debug_light")]
                                        println!("<<< [SUB:{thread_id}] Failed to take response as bytes after failed parsing via starting bytesware");
                                        
                                        Err(ResponseBuilder::new()
                                            .version(Version::CHAT10)
                                            .code(ResponseCode::Error)
                                            .build()
                                            .unwrap()
                                            .as_bytes()
                                            .unwrap()
                                        )
                                    }
                                }
                            }
                        };

                        
                        match req {
                            Ok(req) if req.method == Method::Send => {
                                let second = routes.send.0.middleware(req, state.clone()).await;
                                let res = routes.send.1.bytesware(second).await;
                                match res {
                                    Ok(val) => {
                                        match mp_tx_sub.send(val) {
                                            Ok(_) => {
                                                #[cfg(feature = "debug_full")]
                                                println!("<0> [SUB:{thread_id}] Sent bytes via the channel MPSC");
                                            },
                                            Err(_e) => {
                                                #[cfg(feature = "debug_full")]
                                                println!(">0< [SUB:{thread_id}] Failed to write bytes to the channel with error {_e}");
                                            }
                                        };
                                    },
                                    Err(val) => {
                                        match stream.write(&val).await {
                                            Ok(0) => {
                                                #[cfg(feature = "debug_light")]
                                                println!(">>> [SUB:{thread_id}] Connection closed");

                                                return;
                                            },
                                            Ok(_val) => {
                                                #[cfg(feature = "debug_full")]
                                                println!("--> [SUB:{thread_id}] Wrote {_val} bytes to user");

                                                continue;
                                            },
                                            Err(_e) => {
                                                #[cfg(feature = "debug_light")]
                                                println!("<<< [SUB:{thread_id}] Failed to send error response to user with error {_e}");

                                                continue;
                                            }
                                        }
                                    }
                                } 
                            },
                            Ok(_) => {
                                #[cfg(feature = "debug_light")]
                                println!("--> [SUB:{thread_id}] User cant send this type of requests during handshake");

                                let response = ResponseBuilder::new()
                                    .version(Version::CHAT10)
                                    .code(ResponseCode::Error)
                                    .build()
                                    .unwrap()
                                    .as_bytes()
                                    .unwrap();

                                match stream.write(&response).await {
                                    Ok(0) => {
                                        #[cfg(feature = "debug_light")]
                                        println!(">>> [SUB:{thread_id}] Connection is closed");

                                        return;
                                    },
                                    Ok(_val) => {
                                        #[cfg(feature = "debug_full")]
                                        println!("--> [SUB:{thread_id}] Wrote {_val} bytes to user with error");

                                        continue;
                                    },
                                    Err(_e) => {
                                        #[cfg(feature = "debug_light")]
                                        println!("<<< [SUB:{thread_id}] Failed to write to user with error {_e}");
                                        
                                        continue;
                                    }
                                }
                                    
                            },
                            Err(res) => {
                                #[cfg(feature = "debug_light")]
                                println!("<<< [SUB:{thread_id}] Failed to get request from the starting_bytesware");

                                match stream.write(&res).await {
                                    Ok(0) => {
                                        #[cfg(feature = "debug_light")]
                                        println!("<<< [SUB:{thread_id}] Connection is closed before it should be");

                                        return;
                                    },
                                    Ok(_val) => {
                                        #[cfg(feature = "debug_full")]
                                        println!("--> [SUB:{thread_id}] Wrote {_val} bytes to the user with error");

                                        continue;
                                    },
                                    Err(_e) => {
                                        #[cfg(feature = "debug_light")]
                                        println!("<<< [SUB:{thread_id}] Failed to write to user with error {_e}");
                                        
                                        continue;
                                    }
                                } 
                            }
                        }
                    },
                    Err(_e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [SUB:{thread_id}] Failed to write to user with error: {_e}");
                    }
                }
            },
            val = listener => {
                match val {
                    Ok(val) => {
                        match stream.write(&val).await {
                            Ok(0) => {
                                #[cfg(feature = "debug_light")]
                                println!(">>> [SUB:{thread_id}] Connection is closed");

                                return;
                            },
                            Ok(_val) => {
                                #[cfg(feature = "debug_full")]
                                println!("--> [SUB:{thread_id}] Wrote {_val} bytes to user");
                                
                                continue;
                            },
                            Err(_e)  => {
                                #[cfg(feature = "debug_light")]
                                println!("<<< [SUB:{thread_id}] Failed to write bytes to user with error: {_e}");
                                
                                return;
                            }
                        }
                    },
                    Err(_e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [SUB:{thread_id}] Failed to read from channel with error {_e}");

                        let state = state.lock().await;
                        let app = state.app.lock().await;

                        if let Some(rx) = app.extension.get::<Sender<Message>>() {
                            // lagged
                            let message = Message::Error(thread_id, Utc::now(), ErrorType::Lagged);
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

                        continue;
                    }
                }
            }
        }
    }
}