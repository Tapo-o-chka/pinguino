use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use async_trait::async_trait;

use crate::protocol::request::{ParseError, Request};
use crate::protocol::response::{Response, ResponseCode};
use crate::protocol::utils::set_keepalive;

#[derive(Debug)]
pub struct DefaultClient {
    pub target: SocketAddr,
    pub out_reciever: Arc<Mutex<Receiver<Request>>>,
    pub out_sender: Arc<Sender<Request>>,
    pub in_reciever: Arc<Mutex<UnboundedReceiver<Response>>>,
    pub in_sender: Arc<UnboundedSender<Response>>,
    handle: Option<tokio::task::JoinHandle<Result<(), ()>>>
}

#[allow(dead_code)]
#[async_trait]
pub trait ClientTrait {
    fn new(target: SocketAddr) -> Self;

    async fn bind(&self, name: String) -> Result<String, ClientError>;

    async fn handshake(&mut self, token: String) -> Result<(), ClientError>;

    async fn send(&mut self, message: String) -> Result<(), ClientError>;

    async fn subscribe(&self) -> Arc<Mutex<UnboundedReceiver<Response>>>;

    async fn terminate(&mut self) -> Result<(), ClientError>;
}

#[async_trait]
impl ClientTrait for DefaultClient {
    fn new(target: SocketAddr) -> Self {
        let (out_sender, out_reciever) = mpsc::channel::<Request>(32);
        let (in_sender, in_reciever) = mpsc::unbounded_channel::<Response>();

        DefaultClient {
            target,
            out_reciever: Arc::new(Mutex::new(out_reciever)),
            out_sender: Arc::new(out_sender),
            in_reciever: Arc::new(Mutex::new(in_reciever)),
            in_sender: Arc::new(in_sender),
            handle: None,
        }
    }

    async fn bind(&self, name: String) -> Result<String, ClientError> {
        let mut stream = match TcpStream::connect(self.target).await {
            Ok(val) => val,
            Err(e) => { return Err(ClientError::CouldntConnect(e)); }
        };

        match stream.write(format!("<CHAT \\ 1.0>\n<Method@Bind>\n<Name@{name}>").as_bytes()).await {
            Ok(0) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [BIND] Closed connection, before it needed");

                return Err(ClientError::ClosedConnection);
            },
            Ok(_val) => {
                #[cfg(feature = "debug_full")]
                println!("--> [BIND] Sent {_val} bytes to the server");

                let mut read_buf = [0u8; 512];
                match stream.read(&mut read_buf).await {
                    Ok(0) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [BIND] Closed connection, before it needed");

                        return Err(ClientError::ClosedConnection);                        
                    },
                    Ok(_val) => {
                        #[cfg(feature = "debug_full")]
                        println!("--> [BIND] Read {_val} bytes to the server");

                        let response = match Response::from_bytes(&read_buf) {
                            Ok(val) => val,
                            Err(e) => { 
                                #[cfg(feature = "debug_light")]
                                println!("<<< [BIND] Failed to parse response with error {:?}", e);

                                return Err(ClientError::ParseError(e)); },
                        };
                        if let Some(token) = response.token {
                            return Ok(token)
                        }
                        // No real need for else, right?
                        return Err(ClientError::MissingToken);
                    },
                    Err(e) => {
                        #[cfg(feature = "debug_light")]
                        println!("<<< [BIND] Failed to read from the server with error {e}");

                        return Err(ClientError::ReadingFailed(e));
                    }
                }
            },
            Err(e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [BIND] Failed to send request to the server with error: {e}");

                return Err(ClientError::SendingFailed(e));
            }
        }
    }

    async fn handshake(&mut self, token: String) -> Result<(), ClientError> {
        let stream = match TcpStream::connect(self.target).await {
            Ok(val) => val,
            Err(e) => { 
                #[cfg(feature = "debug_light")]
                println!("<<< [HAND] Failed to connect to the server with error {e}");

                return Err(ClientError::CouldntConnect(e)); 
            }
        };
        
        let mut stream = match set_keepalive(stream).await {
            Ok(val) => val,
            Err(e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [HAND] Failed to failed to start keepalive {e}");

                return Err(ClientError::CouldntConnect(e));
            }
        };

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
                            let handle = tokio::spawn(cool(stream, self.out_reciever.clone(), self.in_sender.clone()));
                            self.handle = Some(handle);
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

    async fn send(&mut self, message: String) -> Result<(), ClientError> {
        let message = message.trim_end_matches('\n');
        let addr = SocketAddr::from_str("127.0.0.1:9999").unwrap(); // Just a place holder.
        let request = match Request::parse(&format!("<CHAT \\ 1.0>\n<Method@Send>\n<Message@'{0}'>", message), Arc::new(addr)) {
            Ok(val) => val,
            Err(e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [SEND] Failed to parse the request with error {:?}", e);
                
                return Err(ClientError::ParseError(e));
            }
        };
        match self.out_sender.send(request).await {
            Ok(_) => Ok(()),
            Err(_e) => {
                #[cfg(feature = "debug_light")]
                println!("<<< [SEND] Failed to send the request with error {_e}");

                Err(ClientError::InternalError)
            }
        }
    }

    async fn subscribe(&self) -> Arc<Mutex<UnboundedReceiver<Response>>> {
        self.in_reciever.clone()
    }

    async fn terminate(&mut self) -> Result<(), ClientError> {
        if let Some(handle) = &self.handle {
            if handle.is_finished() {
                #[cfg(feature = "debug_light")]
                println!("<<< [TERM] Failed to terminate the handle, because its already finished");
                
                self.handle = None;
                return Err(ClientError::AlreadyFinished);
            }

            self.handle = None;
            return Ok(());
        }
        #[cfg(feature = "debug_light")]
        println!("<<< [TERM] No handle is currently running");

        Err(ClientError::NoActiveHandle)
    } 
}

async fn cool(mut stream: TcpStream, out_recieverr: Arc<Mutex<Receiver<Request>>>, in_sender: Arc<UnboundedSender<Response>>) -> Result<(), ()>{
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

#[allow(unused)]
#[derive(Debug)]
pub enum ClientError {
    CouldntConnect(std::io::Error),
    ClosedConnection,
    SendingFailed(std::io::Error),
    ReadingFailed(std::io::Error),
    ParseError(ParseError),
    MissingToken,
    WrongResponseCoce(ResponseCode),
    InternalError,
    NoActiveHandle,
    AlreadyFinished,
}