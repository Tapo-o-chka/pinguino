//! ## `Client`
//! 
//! This module holds main functionality with which client developer will be working the most.
//! For non-custom usage you could just do:
//! 
//! ```
//! let client = Client::default();
//! ```
//! 
//! It would be looking at `127.0.0.1:8080`
//! 
//! ## Example
//! 
//! ```
//! let client = Client::default();
//! 
//! client.bind("Jeff".to_string()).await.unwrap();
//! 
//! client.handshake().await.unwrap();
//! 
//! let sub = client.subscribe().await;
//! tokio::spawn(async move {
//!     let mut reciever = sub.lock().await;
//! 
//!     while let Some(message) = reciever.recv().await {
//!         println!("Got message:\n{0}", message.pretty_string());
//!     }
//! });
//! 
//! client.send("Hello world!".to_string()).await.unwrap();
//! ```
use crate::protocol::request::{ParseError, Request};
use crate::protocol::response::{Response, ResponseCode};
use crate::protocol::Varmap;
use std::any::Any;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use wares::subscribe::{DefaultSubscribe, SubscribeTrait};
use wares::terminate::{TerminateTrait, DefaultTerminate};
use wares::{BindTrait, DefaultBind, DefaultHandshake, DefaultSend, HandshakeTrait, SendTrait};

pub mod wares;

/// ## `ClientState`
/// 
/// This struct is what custom user implementions would be working with. 
/// 
/// for sticky note, use state.varmap
/// 
/// **ATTENTION** Dont forget to `drop()` / or make it be free'ed, so programm wont be dead locked.
/// 
/// ## Example
/// 
/// ```
/// fn some_ware(state: Arc<Mutex<ClientState>>) {
///     let locked = state.lock().await;
/// 
///     locked.varmap.insert("Hello there!");
/// 
///     // Calling this one would dead lock programm. call before locked.drop(), if you need to access it
///     // in different context.
///     /// 
///     // let new_locked = state.lock().await;
/// 
///     // locked would be dropped
/// }
/// ```  
#[derive(Debug)]
pub struct ClientState {
    pub token: Option<String>,
    pub target: SocketAddr,
    pub out_reciever: Arc<Mutex<Receiver<Request>>>,
    pub out_sender: Arc<Sender<Request>>,
    pub in_reciever: Arc<Mutex<UnboundedReceiver<Response>>>,
    pub in_sender: Arc<UnboundedSender<Response>>,
    pub handle: Option<tokio::task::JoinHandle<Result<(), ()>>>,
    pub varmap: Varmap,
}

/// ## `ClientError`
/// 
/// This enum is designed (but poorely) to be the error return by a client.
/// 
/// ## `Example`
/// ```
/// let client = Client::default();
/// 
/// match client.bind("Jeff".to_string()).await.err() {
///     Some(ClientError::SendingFailed(e)) => {
///         panic!("Failed to send bind request to the server {e}")
///     },
///     Some(_) => {
///         panic!("Failed to make bind request to the server, but we are not interested in the exact error")
///     },
///     None => {
///         // it ended without errors
///     }
/// }
/// ```
/// 
/// *Should it be some kind of anyhow error?*
/// *Are there any implementations missing?*
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


/// ## `Client`
/// 
/// This struct holds main functionality of the client. It is similar to other client things and is
/// self sufficient in terms of implementing pre-defined things, that user is expected to be doing.
/// 
/// ## Example
/// ```
/// // Creating default Client
/// let client = Client::default();
/// 
/// // Basic bind request. It saves extracted token in the client.state.
/// client.bind("Jeff".to_string()).await.unwrap();
/// 
/// // Use extracted token to start handshake
/// client.handshake().await.unwrap();
/// 
/// // Send message in the context of the happening handshake
/// client.send("Helo world!".to_string()).await.unwrap();  
/// ```
/// 
/// ## Current problem
/// Current state of `Client` implementation makes me wonder how to do custom methods. And this goal
/// would probably require full re-werite of the client logic, but for now, its not my problem, it is
/// the problem of the future me.
/// 
/// *Comment* for get() function from `varmap`, access `client.state.varmap`
/// yes, it would need to be blocked, but you get the point.
#[derive(Debug)]
pub struct Client {
    sbind: Box<dyn BindTrait>,
    shandshake: Box<dyn HandshakeTrait>,
    ssend: Box<dyn SendTrait>, 
    ssubscribe: Box<dyn SubscribeTrait>,
    sterminate: Box<dyn TerminateTrait>,
    pub state: Arc<Mutex<ClientState>>,
}

impl Client {
    /// `bind()` function is needed to execute pre-defined [`BindTrait`] function either default or custom one provided 
    /// via [`ClientBuilder`].bind().
    /// 
    /// ## Example
    /// ```
    /// let client = Client::default();
    /// 
    /// client.bind("Jeff".to_string()).await.unwrap();
    /// ```
    /// 
    /// *This function is designed to bind token to ther state, if you dont have one via accessing the server*
    pub async fn bind(&self, name: String) -> Result<(), ClientError> {
        self.sbind.bind(self.state.clone(), name).await
    }

    /// `bindt()` function is needed to execute pre-defined [`BindTrait`] function either default or custom one provided
    /// via [`ClientBuilder`].bind()
    /// 
    /// ## Example
    /// ```
    /// let client = Client::default();
    /// 
    /// client.tbind("*Some token*".to_string()).await.unwrap();
    /// ```
    /// 
    /// ## If you are wondering how to get the token
    /// ```
    /// let client = Client::default();
    /// 
    /// client.bind("Jeff".to_string()).await.unwrap();
    /// 
    /// let locked = client.state.lock().await;
    /// 
    /// printf("My token: {0}", locked.token.unwrap());
    /// ```
    /// 
    /// *This function is designed to bind token to the state, if you have it without accessing the server*
    pub async fn bindt(&self, token: String) {
        self.sbind.bindt(self.state.clone(), token).await;
    }

    /// `handshake()` function is needed to execute pre-defined [`HandshakeTrait`] function either default or custom one
    /// provided via [`ClientBuilder`].handshake().
    /// 
    /// ## Example
    /// ```
    /// let client = Client::default();
    /// 
    /// client.bind("Jeff".to_string()).await.unwrap();
    /// 
    /// // Calling `client.handshake()` should not be possible, without "token"
    /// client.handshake().await.unwrap();
    /// ```
    pub async fn handshake(&self) -> Result<(), ClientError> {
        self.shandshake.handshake(self.state.clone()).await
    }

    /// `send()` function is needed to execute pre-defined [`SendTrait`] function either default or custom one
    /// provided via [`ClientBuilder`].send()
    /// 
    /// ## Example
    /// ```
    /// let client = Client::default();
    /// 
    /// client.bind("Jeff".to_string()).await.unwrap();
    /// 
    /// client.handshake().await.unwrap();
    /// 
    /// client.send("Hello world!").await.unwrap();
    /// ```
    pub async fn send(&self, message: String) -> Result<(), ClientError> {
        self.ssend.send(self.state.clone(), message).await
    }

    /// `subscirbe()` function is needed to execute pre-defined [`SendTrait`] function either default or custom one
    /// provided via [`ClientBuilder`].subscribe()
    /// 
    /// ## Example
    /// ```
    /// let client = Client::default();
    /// 
    /// client.bind("Jeff".to_string()).await.unwrap();
    /// 
    /// client.handshake().await.unwrap();
    /// 
    /// let sub = client.subscribe().await;
    /// 
    /// tokio::spawn(async move {
    ///     let mut reciever = sub.lock().await;
    /// 
    ///     while let Some(message) = reciever.recv().await {
    ///         println!("Got message:\n{0}", message.pretty_string());
    ///     }
    /// }).await;
    /// ```
    pub async fn subscribe(&self) -> Arc<Mutex<UnboundedReceiver<Response>>> {
        self.ssubscribe.subscribe(self.state.clone()).await
    }

    /// `terminate()` function is needed to execute pre-defined [`SendTrait`] function either default or custom one
    /// provided via [`ClientBuilder`].subscribe()
    /// 
    /// ## Example
    /// ```
    /// let client = Client::default();
    /// 
    /// client.bind("Jeff".to_string()).await.unwrap();
    /// 
    /// client.handshake().await.unwrap();
    /// 
    /// let sub = client.subscribe().await;
    /// 
    /// tokio::spawn(async move {
    ///     let mut reciever = sub.lock().await;
    /// 
    ///     while let Some(message) = reciever.recv().await {
    ///         println!("Got message:\n{0}", message.pretty_string());
    ///     }
    ///     println!("Ended listening!");
    /// });
    /// 
    /// // Lets sleep, so task would spawn correctly
    /// tokio::time::sleep(Duration::from_secs(3)).await;
    /// 
    /// // For example lets send some messages to see that it prints
    /// client.send("Hello world <1>!".to_string()).await.unwrap();
    /// client.send("Hello world <2>!".to_string()).await.unwrap();
    /// client.send("Hello world <3>!".to_string()).await.unwrap();
    /// 
    /// // Lets now terminate
    /// client.terminate().await.unwrap();
    /// // Here should be printed `Ended listening!`
    /// 
    /// // Now after that send, we dont recieve any message
    /// client.send("Hello world <1>!".to_string()).await.unwrap();
    /// ```
    pub async fn terminate(&self) -> Result<(), ClientError> {
        self.sterminate.terminate(self.state.clone()).await
    }

    /// `insert()` This function is needed to easily insert value into `client.state.varmap`
    /// without 
    pub async fn insert<T: Any + Send + Sync>(&self, val: T) {
        let mut locked = self.state.lock().await;

        locked.varmap.insert(val);
    }
}

/// ## `ClientBuilder`
/// 
/// Just a builder pattern with inserting values into `client.state.varmap` before it is built
/// 
/// ## Examples
/// ```
/// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
///     .insert("Cool message")
///     .build();
/// ```
#[derive(Debug)]
pub struct ClientBuilder {
    sbind: Box<dyn BindTrait>,
    shandshake: Box<dyn HandshakeTrait>,
    ssend: Box<dyn SendTrait>, 
    ssubscribe: Box<dyn SubscribeTrait>,
    sterminate: Box<dyn TerminateTrait>,
    pub state: Arc<Mutex<ClientState>>,
}

impl ClientBuilder {
    /// `ClientBuilder::new()` creates new instance of the `ClientBuilder`
    /// 
    /// ## Example
    /// 
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .build();
    /// ```
    pub fn new(target: SocketAddr, capacity: Option<usize>) -> Self {
        ClientBuilder {
            sbind: Box::new(DefaultBind),
            shandshake: Box::new(DefaultHandshake),
            ssend: Box::new(DefaultSend),
            ssubscribe: Box::new(DefaultSubscribe),
            sterminate: Box::new(DefaultTerminate),
            state: Arc::new(Mutex::new(ClientState::new(target, capacity)))
        }
    }

    /// `ClientBuilder::bind()` sets custom `bind` ware instead of the current one.
    /// 
    /// This function takes `struct` that implements [`BindTrait`] as input.
    /// 
    /// ## Example
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .bind(MyCustomBindImplementator) // it implements `BindTrait` tho.
    ///     .build();
    /// ```
    pub fn bind(mut self, bind: Box<dyn BindTrait>) -> Self {
        self.sbind = bind;
        self
    }

    /// `ClientBuilder::handshake()` sets custom `handshake` ware instead of the current one.
    /// 
    /// This function takes `struct` that implements [`HandshakeTrait`] as a parametr.
    /// 
    /// ## Example
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .handshake(MyCustomHandshakeImplementator) // it implements `HandshakeTrait` tho.
    ///     .build();
    /// ```
    pub fn handshake(mut self, handshake: Box<dyn HandshakeTrait>) -> Self {
        self.shandshake = handshake;
        self
    }

    /// `ClientBuilder::send()` sets custom `send` ware instead of the current one.
    /// 
    /// This function takes `struct` that implements [`SendTrait`] as a parametr.
    /// 
    /// ## Example
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .send(MyCustomSendImplementator) // it implements `SendTrait` tho.
    ///     .build();
    /// ```
    pub fn send(mut self, send: Box<dyn SendTrait>) -> Self {
        self.ssend = send;
        self
    }

    /// `ClientBuilder::subscribe()` sets custom `subscribe` function instead of the current one.
    /// 
    /// This function takes `struct` that implements [`SubscribeTrait`] as a parametr.
    /// 
    /// ## Example
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .subsribe(MyCustomSubscribeImplementator) // it implements `SubscribeTrait` tho.
    ///     .build();    
    /// ```
    pub fn subscribe(mut self, subscribe: Box<dyn SubscribeTrait>) -> Self {
        self.ssubscribe = subscribe;
        self
    }

    /// `ClientBuilder::terminate()` sets custom `terminate` function instead of the current one
    /// 
    /// This function takes `struct` that implements [`TerminateTrait`] as a parametr.
    /// 
    /// ## Example
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .terminate(MyCustomTerminateImplementator) // it implements `TerminateTrait` tho.
    ///     .build();    
    /// ```
    pub fn terminate(mut self, terminate: Box<dyn TerminateTrait>) -> Self {
        self.sterminate = terminate;
        self
    }

    /// `ClientBuilder::insert()` inserts value into `state.varmap` before [`Client`] is built.
    /// 
    /// ## Example
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .insert("Cool message")
    ///     .build();
    /// ```
    /// It is the equivalent of:
    /// ```
    /// let client = Client::default();
    /// client.insert("Cool message");
    /// ``` 
    pub async fn insert<T: Any + Send + Sync>(&self, val: T) {
        let mut locked = self.state.lock().await;

        locked.varmap.insert(val);
    }

    /// `ClientBuilder::build()` finishes builder pattern and returns built Client.
    /// 
    /// ## Example
    /// ```
    /// let client = ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None)
    ///     .build();
    /// ```
    pub fn build(self) -> Client {
        Client {
            sbind: self.sbind,
            shandshake: self.shandshake,
            ssend: self.ssend,
            ssubscribe: self.ssubscribe,
            sterminate: self.sterminate,
            state: self.state
        }
    }
}

impl Default for Client {
    /// Just default values for [`Client`]. Sets `target` field to `127.0.0.1:8080` and `mpsc` channel capacity to 32.
    fn default() -> Self {
        ClientBuilder::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None).build()
    }
}

impl ClientState {
    /// Well, just new state. Nothing too special. Developer wont be needing this, it is
    /// for the sake of readability of the source code, nothing more.
    /// 
    /// ## Example
    /// ```
    /// let state = ClientState::new(SocketAddr::from_str("127.0.0.1:8080").unwrap(), None);
    /// ```
    fn new(target: SocketAddr, capacity: Option<usize>) -> Self {
        let cap = capacity.unwrap_or(32);
        let (out_sender, out_reciever) = mpsc::channel::<Request>(cap);
        let (in_sender, in_reciever) = mpsc::unbounded_channel::<Response>();

        ClientState {
            token: None,
            target,
            out_reciever: Arc::new(Mutex::new(out_reciever)),
            out_sender: Arc::new(out_sender),
            in_sender: Arc::new(in_sender),
            in_reciever: Arc::new(Mutex::new(in_reciever)),
            handle: None,
            varmap: Varmap::new()
        }
    }
}