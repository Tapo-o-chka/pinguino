use crate::protocol::request::{RawRequest, Version};
use crate::protocol::response::{Response, ResponseCode};
use crate::protocol::utils::set_keepalive;
use tokio::sync::Mutex;
use tokio::task::Id;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio::sync::{mpsc::UnboundedSender as MpscSender, broadcast::Receiver as BrReceiver};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::protocol::request::Method;
use crate::protocol::response::ResponseBuilder;

use super::{Routes, State, RouteRes, send_handler::handle_send};

pub async fn handle_request(routes: Arc<Routes>, req_bytes: [u8; 512], addr: Arc<SocketAddr>, state: Arc<Mutex<State>>, _thread_id: Id) -> RouteRes {
    let raw_req = RawRequest {
        bytes: req_bytes,
        addr: addr,
    };

    let req_res = routes.starting_bytesware.bytesware(raw_req).await;    

    let req = match req_res {
        Ok(val) => val,
        Err(res) => {
            #[cfg(feature = "debug_light")]
            println!("<<< [SUB:{_thread_id}] Failed to get parse request via bytesware");
            match res.as_bytes() {
                Ok(val) => {
                    return RouteRes::None(Err(val));
                },
                Err(_) => {
                    return RouteRes::None(
                        Err(ResponseBuilder::new()
                            .version(Version::CHAT10)
                            .code(ResponseCode::Error)
                            .build()
                            .unwrap()
                            .as_bytes()
                            .unwrap()
                        )
                    );
                }
            }
        }
    };

    match req.method {
        Method::Bind => {
            let second_res: Result<Response, Response> = routes.bind.0.middleware(req, state).await;
            RouteRes::Bind(routes.bind.1.bytesware(second_res).await)
        },
        Method::Send => {
            let second_res: Result<Response, Response> = routes.send.0.middleware(req, state).await;
        
            RouteRes::Send(routes.send.1.bytesware(second_res).await)
        },
        Method::Handshake => {
            let second_res = routes.handshake.0.middleware(req, state).await;
        
            RouteRes::Handshake(routes.handshake.1.bytesware(second_res).await)
        }
    }
}

pub async fn handle_request1(routes: Arc<Routes>, mut stream: TcpStream, addr: Arc<SocketAddr>, state: Arc<Mutex<State>>, br_tx_sub: BrReceiver<[u8; 512]>, mp_tx_sub: MpscSender<[u8; 512]>) {
    let thread_id = tokio::task::id();
    let mut blocked = state.lock().await;
    blocked.varmap.insert(thread_id.clone());
    drop(blocked);

    #[cfg(feature = "debug_light")]
    println!(">>> [SUB:{thread_id}] Recieved request from {addr}");

    let mut buf = [0u8; 512];
    match stream.read(&mut buf).await {
        Ok(0) => {
            #[cfg(feature = "debug_light")]
            println!(">>> [SUB:{thread_id}] Connection closed by user");
            
            return;
        },
        Ok(_val) => {
            #[cfg(feature = "debug_full")]
            println!("--> [SUB:{thread_id}] Read {_val} bytes from user");
            let res = handle_request(routes.clone(), buf, addr.clone(), state.clone(), thread_id).await;

            match res {
                RouteRes::Bind(val) => {
                    let write_buf = match val {
                        Ok(val) => val,
                        Err(val) => val,
                    };

                    match stream.write(&write_buf).await {
                        Ok(0) => {
                            #[cfg(feature = "debug_light")]
                            println!(">>> [SUB:{thread_id}] Conncetion closed");

                            return;
                        },
                        Ok(_val) => {
                            #[cfg(feature = "debug_full")]
                            println!("--> [SUB:{thread_id}] Wrote {_val} to user");
                        },
                        Err(_e) => {
                            #[cfg(feature = "debug_full")]
                            println!("<<< [SUB:{thread_id}] Failed to write to user with error: {_e}")
                        }
                    }
                },
                RouteRes::Send(_) => {
                        let response = ResponseBuilder::new()
                            .version(Version::CHAT10)
                            .code(ResponseCode::Unauthorized)
                            .build()
                            .unwrap()
                            .as_bytes()
                            .unwrap();

                        match stream.write(&response).await {
                            Ok(0) => {
                                #[cfg(feature = "debug_light")]
                                println!(">>> [SUB:{thread_id}] Connection closed");

                                return;
                            },
                            Ok(_) => {},
                            Err(_e) => {
                                #[cfg(feature = "debug_full")]
                                println!("<<< [SUB:{thread_id}] Failed to write error response to user with error {_e}");
                            }
                        }    
                    },
                RouteRes::Handshake(val) => {
                    match val {
                        Ok(val) => {
                            let mut stream = match set_keepalive(stream).await {
                                Ok(val)=> val,
                                Err(_e) => {
                                    #[cfg(feature = "debug_full")]
                                    println!("<<< [SUB:{thread_id}] Faileld to start keepalive, panic! {_e}");
                                    
                                    return;
                                }
                            };

                            match stream.write(&val).await {
                                Ok(0) => {
                                    #[cfg(feature = "debug_light")]
                                    println!(">>> [SUB:{thread_id}] Connection is closed");

                                    return;
                                },
                                Ok(_val) => {
                                    #[cfg(feature = "debug_full")]
                                    println!("--> [SUB:{thread_id}] Wrote {_val} bytes to user");
                                    
                                    handle_send(stream, routes.clone(), addr.clone(), state.clone(), br_tx_sub, mp_tx_sub, thread_id).await;
                                },
                                Err(_e) => {
                                    #[cfg(feature = "debug_full")]
                                    println!("<<< [SUB:{thread_id}] Failed to write to user with error {_e}");
                                    
                                    return;
                                }
                            }
                        },
                        Err(val) => {
                            match stream.write(&val).await {
                                Ok(0) => {
                                    #[cfg(feature = "debug_light")]
                                    println!(">>> [SUB:{thread_id}] Connection was closed before closing");

                                    return;
                                },
                                Ok(_val) => {
                                    #[cfg(feature = "debug_full")] 
                                    println!("--> [SUB:{thread_id}] Wrote {_val} bytes to the user, but closing"); 
                                },
                                Err(_e) => {
                                    #[cfg(feature = "debug_full")]
                                    println!("<<< [SUB:{thread_id}] Failed to write to user with error: {_e}");
                                    return;
                                }
                            }
                        }
                    }                    
                },
                RouteRes::None(val) => {
                    let resp = match val {
                        Ok(val) => val,
                        Err(val) => val
                    };

                    match stream.write(&resp).await {
                        Ok(0) => {
                            #[cfg(feature = "debug_light")]
                            println!(">>> [SUB:{thread_id}] Connection is closed");

                            return;
                        },
                        Ok(_val) => {
                            #[cfg(feature = "debug_full")]
                            println!("--> [SUB:{thread_id}] Wrote {_val} bytes to user");
                            
                            return;
                        },
                        Err(_e) => {
                            #[cfg(feature = "debug_full")]
                            println!("<<< [SUB:{thread_id}] Failed to write to user with error {_e}");

                            return;
                        }
                    }
                },
            }
        },
        Err(_e) => {
            #[cfg(feature = "debug_full")]
            println!("<<< [SUB:{thread_id}] Failed to read from user with error {_e}");

            return;
        }
    }   


}

pub async fn handle_wrapper(routes: Arc<Routes>, stream: TcpStream, state: Arc<Mutex<State>>, addr: Arc<SocketAddr>, br_tx_sub: BrReceiver<[u8; 512]>, mp_tx_sub: MpscSender<[u8; 512]>) {
    handle_request1(routes, stream, addr, state.clone(), br_tx_sub, mp_tx_sub).await;

    let locked = state.lock().await;
    if let Some(after) = locked.after.as_deref() {
        after.execute(state.clone()).await;
    }
}