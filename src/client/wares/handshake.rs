use std::sync::Arc;
use std::fmt::Debug;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, select, sync::{mpsc::{Receiver, UnboundedSender}, Mutex}};

use crate::{client::{ClientError, ClientState}, protocol::{request::Request, response::{Response, ResponseCode}, set_keepalive}};

/// ## `DefaultHandshake`
/// 
/// This is the default implementator of the [`HandshakeTrait`]. It consists of the general functionality 
/// needed to start `Handshake` with the server - recieving token -> creating handler -> Letting user know if
/// it failed or not.
/// 
/// ## `Help on default implementatiin`
/// 
/// please, consider looking at the source code.
#[derive(Debug)]
pub struct DefaultHandshake;

/// ## `Handshake`
/// 
/// This trait is responsible for handling start of the `Handshake`between client and the server.
/// 
/// ## How it look in human way
/// 
/// ```
/// #[async_trait::async_trait]
/// pub trait HandshakeTrait {
///     async fn handshake(state: Arc<Mutex<ClientState>>) -> Result<(), ClientError>;
/// }
/// ```
#[async_trait::async_trait]
pub trait HandshakeTrait: Debug + Send + Sync {
    async fn handshake(&self, state: Arc<Mutex<ClientState>>) -> Result<(), ClientError>;
}

#[async_trait::async_trait]
impl HandshakeTrait for DefaultHandshake {
    async fn handshake(&self, state: Arc<Mutex<ClientState>>) -> Result<(), ClientError> {
        let mut locked = state.lock().await;
        // If no token found -> problem.
        let token = if let Some(val) = &locked.token {
            val
        } else {
            return Err(ClientError::MissingToken);
        };

        // Connecting
        let stream = match TcpStream::connect(locked.target).await {
            Ok(val) => val,
            Err(e) => { 
                #[cfg(feature = "debug_light")]
                println!("<<< [HAND] Failed to connect to the server with error {e}");

                return Err(ClientError::CouldntConnect(e)); 
            }
        };
        
        // Setting keepalive
        let mut stream = match set_keepalive(stream).await {
            Ok(val) => val,
            Err(e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [HAND] Failed to failed to start keepalive {e}");

                return Err(ClientError::CouldntConnect(e));
            }
        };

        // Sending the request
        //
        // If there is a need in custom EndingBytesware for the Client you could create function and call it here
        // before sending.
        match stream.write(format!("<CHAT \\ 1.0>\n<Method@Handshake>\n<Authorization@'{token}'>").as_bytes()).await {
            Ok(0) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [HAND] Connection closed be it should've");

                return Err(ClientError::ClosedConnection);
            },
            Ok(_val) => {
                #[cfg(feature = "debug_full")]
                println!("--> [HAND] Sent {_val} bytes to the server");

                let mut read_buf = [0u8; 512];
                match stream.read(&mut read_buf).await {
                    Ok(0) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [HAND] Connection closed before it should've");

                        return Err(ClientError::ClosedConnection);
                    },
                    Ok(_val) => {
                        #[cfg(feature = "debug_full")]
                        println!("--> [HAND] Read {_val} bytes from the server");

                        let response = match Response::from_bytes(&read_buf) {
                            Ok(val) => val,
                            Err(e) => {
                                #[cfg(feature = "debug_light")]
                                println!("<<< [HAND] Failed to parse response from bytes with error {:?}", e);

                                return Err(ClientError::ParseError(e));
                            }
                        };

                        if response.code == ResponseCode::AuthOK {
                            let handle = tokio::spawn(event_loop(stream, locked.out_reciever.clone(), locked.in_sender.clone()));
                            locked.handle = Some(handle);
                            return Ok(());
                        }
                        #[cfg(feature = "debug_light")]
                        println!("<<< [HAND] Wrong response code occured {:?}", response.code);
                        return Err(ClientError::WrongResponseCoce(response.code));
                    },
                    Err(e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [HAND] Failed to read bytes from the server with error {e}");

                        return Err(ClientError::ReadingFailed(e));
                    }
                }
            }
            Err(e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [HAND] Failed to send a request to the server with error {e}");

                return Err(ClientError::SendingFailed(e))
            }
        }
    }
}

pub async fn event_loop(mut stream: TcpStream, out_recieverr: Arc<Mutex<Receiver<Request>>>, in_sender: Arc<UnboundedSender<Response>>) -> Result<(), ()>{
    let mut out_reciever = out_recieverr.lock().await;
    let mut read_buf = [0u8; 512];
    loop {
        select! {
            _ = stream.read(&mut read_buf) => {
                let response = match Response::from_bytes(&read_buf) {
                    Ok(val) => val,
                    Err(_e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [SUBH] Failed to parse Response from bytes with error {:?}", _e);

                        return Err(());
                    }
                };
                match in_sender.send(response) {
                    Ok(_) => {
                        #[cfg(feature = "debug_full")]
                        println!("--> [SUBH] Sent response to in_sender")
                    }
                    Err(_e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [SUBH] Failed to send via in_sender with error {_e}");
                        continue;
                    }
                }
            },
            val = out_reciever.recv() => {
                if let Some(val) = val {
                    let bytes = match val.as_bytes() {
                        Ok(val) => {
                            #[cfg(feature = "debug_full")]
                            println!("--> [SUBH] Parsed recieved request from out_reciever to bytes");
                            
                            val
                        },
                        Err(_) => {
                            #[cfg(feature = "debug_light")]
                            println!("<<< [SUBH] Failed to parse request to bytes recieved from out_reciever");
                            return Err(());
                        }
                    };

                    match stream.write(&bytes).await {
                        Ok(0) => {
                            #[cfg(feature = "debug_light")]
                            println!("<<< [SUBH] Connection closed");
                            return Err(());
                        },
                        Ok(_val) => {
                            #[cfg(feature = "debug_full")]
                            println!("--> [SUBH] Wrote {_val} bytes to the server");
                            continue;
                        },
                        Err(_e) => {
                            #[cfg(feature = "debug_light")]
                            println!("<<< [SUBH] Error occured while writing request to the server with: {_e}");
                            return Err(());
                        }
                    }
                }
            }
        }
    }
}