use crate::protocol::request::{RawRequest, Version};
use crate::protocol::response::{Response, ResponseCode};
use crate::protocol::set_keepalive;
use tokio::sync::Mutex;
use tokio::task::Id;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio::sync::{mpsc::UnboundedSender as MpscSender, broadcast::Receiver as BrReceiver};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::protocol::request::Method;
use crate::protocol::response::ResponseBuilder;

use super::{Routes, State, RouteRes, send_handler::handle_send};

// This function is where Request is processed
pub async fn handle_request(routes: Arc<Routes>, req_bytes: [u8; 512], addr: Arc<SocketAddr>, state: Arc<Mutex<State>>, _thread_id: Id, is_handshake: bool) -> RouteRes {
    // We have RawRequest for easier tossing around bytes and SocketAddr, if user would like to save it.
    let raw_req = RawRequest {
        bytes: req_bytes,
        addr: addr,
    };

    // We have same StartingBytesware for every Method, because we cant extract Method if user uses custom StartingBytesware
    let req_res = routes.starting_bytesware.bytesware(state.clone(), raw_req).await;    

    // Here, if StartingBytesware returned Err(Response) we should send it right away.
    let req = match req_res {
        Ok(val) => val,
        Err(res) => {
            #[cfg(feature = "debug_light")]
            println!("<<< [SUB:{_thread_id}] Failed to get parse request via bytesware");
            
            // I almosst forgot, that we need to route it to the EndingBytesware
            return RouteRes::None(routes.send.1.bytesware(state, Err(res)).await);
        }
    };

    // If everything went OK, we just proceed depending on the method
    match req.method {
        Method::Bind => {
            let second_res: Result<Response, Response> = routes.bind.0.middleware(req, state.clone()).await;

            RouteRes::Bind(routes.bind.1.bytesware(state.clone(), second_res).await)
        },
        Method::Send => {
            if is_handshake {
                let second_res: Result<Response, Response> = routes.send.0.middleware(req, state.clone()).await;
            
                RouteRes::Send(routes.send.1.bytesware(state.clone(), second_res).await)
            } else {
                let res = ResponseBuilder::new()
                    .version(Version::CHAT10)
                    .code(ResponseCode::Unauthorized)
                    .build()
                    .unwrap();

                RouteRes::Send(routes.send.1.bytesware(state.clone(), Err(res)).await)
            }
        },
        Method::Handshake => {
            let second_res = routes.handshake.0.middleware(req, state.clone()).await;
        
            RouteRes::Handshake(routes.handshake.1.bytesware(state.clone(), second_res).await)
        }
    }
}

/// This function is used to route request and send responses.
pub async fn handle_request1(routes: Arc<Routes>, mut stream: TcpStream, addr: Arc<SocketAddr>, state: Arc<Mutex<State>>, br_tx_sub: BrReceiver<[u8; 512]>, mp_tx_sub: MpscSender<[u8; 512]>) {
    // Getting thread_id for better debugging experience, so there wont be the mess
    let thread_id = tokio::task::id();

    // This lock is probably not that blocking, because we handle 1 task per client, so there is not race for state
    let mut blocked = state.lock().await;
    blocked.varmap.insert(thread_id.clone());

    // Not forgetting to drop it, so there wont be infinit lock()
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

            // If we read, and there is something, just handle the request.
            let res = handle_request(routes.clone(), buf, addr.clone(), state.clone(), thread_id, false).await;

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
                RouteRes::Send(val) => {
                    #[cfg(feature = "debug_light")]
                    println!("<<< [SUB:{thread_id}] User shouldnt use Method::Send outside of the Handshake.");

                    // We got 100% error message, because of is_handshake = false
                    match stream.write(&val.err().unwrap()).await {
                        Ok(0) => {
                            #[cfg(feature = "debug_light")]
                            println!(">>> [SUB:{thread_id}] Connection closed");

                            return;
                        },
                        Ok(_val) => {
                            #[cfg(feature = "debug_full")]
                            println!("--> [SUB:{thread_id}] Wrote {_val} bytes to user.");
                        },
                        Err(_e) => {
                            #[cfg(feature = "debug_light")]
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
                                    #[cfg(feature = "debug_light")]
                                    println!("<<< [SUB:{thread_id}] Faileld to start keepalive, panic! {_e}");
                                    
                                    return;
                                }
                            };

                            match stream.write(&val).await {
                                Ok(0) => {
                                    #[cfg(feature = "debug_light")]
                                    println!("<<< [SUB:{thread_id}] Connection is closed");

                                    return;
                                },
                                Ok(_val) => {
                                    #[cfg(feature = "debug_full")]
                                    println!("--> [SUB:{thread_id}] Wrote {_val} bytes to user");

                                    // Here we start Handshake, and accepting only Method::Send from now on from this client     
                                    handle_send(stream, routes.clone(), addr.clone(), state.clone(), br_tx_sub, mp_tx_sub, thread_id).await;
                                },
                                Err(_e) => {
                                    #[cfg(feature = "debug_light")]
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
                                    #[cfg(feature = "debug_light")]
                                    println!("<<< [SUB:{thread_id}] Failed to write to user with error: {_e}");
                                    return;
                                }
                            }
                        }
                    }                    
                },
                RouteRes::None(val) => {
                    // Well, if we didnt love the initial request, we are here.
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
                            #[cfg(feature = "debug_light")]
                            println!("<<< [SUB:{thread_id}] Failed to write to user with error {_e}");

                            return;
                        }
                    }
                },
            }
        },
        Err(_e) => {
            #[cfg(feature = "debug_light")]
            println!("<<< [SUB:{thread_id}] Failed to read from user with error {_e}");

            return;
        }
    }   


}

/// This function is designed to make it more clear, that AfterConnect is used.
pub async fn handle_wrapper(routes: Arc<Routes>, stream: TcpStream, state: Arc<Mutex<State>>, addr: Arc<SocketAddr>, br_tx_sub: BrReceiver<[u8; 512]>, mp_tx_sub: MpscSender<[u8; 512]>) {
    handle_request1(routes, stream, addr, state.clone(), br_tx_sub, mp_tx_sub).await;

    let locked = state.lock().await;
    if let Some(after) = locked.after.as_deref() {
        after.execute(state.clone()).await;
    }
}