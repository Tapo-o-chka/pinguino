use crate::protocol::request::{RawRequest, Version};
use crate::protocol::response::ResponseCode;
use tokio::select;
use tokio::sync::Mutex;
use tokio::task::Id;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio::sync::{mpsc::UnboundedSender as MpscSender, broadcast::Receiver as BrReceiver};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::protocol::request::Method;
use crate::protocol::response::ResponseBuilder;

use super::{Routes, State};

/// This function is designed to be used in the context of Handshake
/// I dont really know what to write here, but TODO: finish commenting this struct
pub async fn handle_send(mut stream: TcpStream, routes: Arc<Routes>, addr: Arc<SocketAddr>, state: Arc<Mutex<State>>, mut br_tx_sub: BrReceiver<[u8; 512]>, mp_tx_sub: MpscSender<[u8; 512]>, _thread_id: Id) {
    let mut buf = [0u8; 512];

    // Making infinit loop, because right now we are waiting for the live connected device
    loop {
        let reader = stream.read(&mut buf);
        let listener = br_tx_sub.recv();

        select! {
            // First is the reader - we wait, until there is incoming request
            // (if is very clear, but still want to point it out)
            val = reader => {
                match val {
                    Ok(0) => {
                        #[cfg(feature = "debug_light")]
                        println!(">>> [SUB:{_thread_id}] Connection closed");

                        return;
                    },
                    Ok(_val) => {
                        #[cfg(feature = "debug_full")]
                        println!("--> [SUB:{_thread_id}] Read {_val} bytes from the user");

                        let raw_req = RawRequest {
                            bytes: buf,
                            addr: addr.clone(),
                        };
                        
                        // Initiall working with the request
                        let req_res = routes.starting_bytesware.bytesware(state.clone(), raw_req).await;    

                        let req = match req_res {
                            Ok(val) => Ok(val),
                            Err(res) => {
                                #[cfg(feature = "debug_light")]
                                println!("<<< [SUB:{_thread_id}] Failed to get parse request via bytesware");
                                
                                let res = routes.send.1.bytesware(state.clone(), Err(res)).await;

                                Err(res.err().unwrap())
                            }
                        };

                        
                        match req {
                            Ok(req) if req.method == Method::Send => {
                                let second = routes.send.0.middleware(req, state.clone()).await;
                                let res = routes.send.1.bytesware(state.clone(), second).await;
                                match res {
                                    Ok(val) => {
                                        match mp_tx_sub.send(val) {
                                            Ok(_) => {
                                                #[cfg(feature = "debug_full")]
                                                println!("<0> [SUB:{_thread_id}] Sent bytes via the channel MPSC");
                                            },
                                            Err(_e) => {
                                                #[cfg(feature = "debug_full")]
                                                println!(">0< [SUB:{_thread_id}] Failed to write bytes to the channel with error {_e}");
                                            }
                                        };
                                    },
                                    Err(val) => {
                                        match stream.write(&val).await {
                                            Ok(0) => {
                                                #[cfg(feature = "debug_light")]
                                                println!(">>> [SUB:{_thread_id}] Connection closed");

                                                return;
                                            },
                                            Ok(_val) => {
                                                #[cfg(feature = "debug_full")]
                                                println!("--> [SUB:{_thread_id}] Wrote {_val} bytes to user");

                                                continue;
                                            },
                                            Err(_e) => {
                                                #[cfg(feature = "debug_light")]
                                                println!("<<< [SUB:{_thread_id}] Failed to send error response to user with error {_e}");

                                                continue;
                                            }
                                        }
                                    }
                                } 
                            },
                            Ok(_) => {
                                #[cfg(feature = "debug_light")]
                                println!("--> [SUB:{_thread_id}] User cant send this type of requests during handshake");

                                let response = ResponseBuilder::new()
                                    .version(Version::CHAT10)
                                    .code(ResponseCode::Error)
                                    .build()
                                    .unwrap();

                                let res = routes.send.1.bytesware(state.clone(), Err(response)).await;

                                // Unwrap is fine, because we defined response as Err 2 lines above
                                match stream.write(&res.err().unwrap()).await {
                                    Ok(0) => {
                                        #[cfg(feature = "debug_light")]
                                        println!(">>> [SUB:{_thread_id}] Connection is closed");

                                        return;
                                    },
                                    Ok(_val) => {
                                        #[cfg(feature = "debug_full")]
                                        println!("--> [SUB:{_thread_id}] Wrote {_val} bytes to user with error");

                                        continue;
                                    },
                                    Err(_e) => {
                                        #[cfg(feature = "debug_light")]
                                        println!("<<< [SUB:{_thread_id}] Failed to write to user with error {_e}");
                                        
                                        continue;
                                    }
                                }
                                    
                            },
                            Err(res) => {
                                #[cfg(feature = "debug_light")]
                                println!("<<< [SUB:{_thread_id}] Failed to get request from the starting_bytesware");

                                match stream.write(&res).await {
                                    Ok(0) => {
                                        #[cfg(feature = "debug_light")]
                                        println!("<<< [SUB:{_thread_id}] Connection is closed before it should be");

                                        return;
                                    },
                                    Ok(_val) => {
                                        #[cfg(feature = "debug_full")]
                                        println!("--> [SUB:{_thread_id}] Wrote {_val} bytes to the user with error");

                                        continue;
                                    },
                                    Err(_e) => {
                                        #[cfg(feature = "debug_light")]
                                        println!("<<< [SUB:{_thread_id}] Failed to write to user with error {_e}");
                                        
                                        continue;
                                    }
                                } 
                            }
                        }
                    },
                    Err(_e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [SUB:{_thread_id}] Failed to write to user with error: {_e}");
                    }
                }
            },
            val = listener => {
                match val {
                    Ok(val) => {
                        match stream.write(&val).await {
                            Ok(0) => {
                                #[cfg(feature = "debug_light")]
                                println!(">>> [SUB:{_thread_id}] Connection is closed");

                                return;
                            },
                            Ok(_val) => {
                                #[cfg(feature = "debug_full")]
                                println!("--> [SUB:{_thread_id}] Wrote {_val} bytes to user");
                                
                                continue;
                            },
                            Err(_e)  => {
                                #[cfg(feature = "debug_light")]
                                println!("<<< [SUB:{_thread_id}] Failed to write bytes to user with error: {_e}");
                                
                                return;
                            }
                        }
                    },
                    Err(_e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [SUB:{_thread_id}] Failed to read from channel with error {_e}");

                        continue;
                    }
                }
            }
        }
    }
}