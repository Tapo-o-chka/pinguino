//! # Router
//! [`Router`] is the key part of `Pinguino` server. It highly relies on `tokio` and `socket2` for providing connection.
//! You can implement custom [`Bytesware`] and [`Middleware`] and pass it in the right field of the [`RouterBuilder`].
//! ## This is the most basic example of the server.
//! If you want just the built-in version, without any adding any additional features, then you could just do as in the first example.
//! ```
//! use pinguino::router::{Router, RouterBuilder};
//! 
//! #[tokio::main]
//! async fn main(){
//!     let router: Router = RouterBuilder::new()
//!         .build();
//! 
//!     router.run().await
//! }
//! ```
//! 
//! ## Using the [`RouterBuilder`] for router creation.
//! If there is need for custom [`Bytesware`] / [`Middleware`], or some shared resource - it could be done as simple as in the following example.
//! ```
//! use pinguino::router::{Router, RouterBuilder};
//!
//! #[tokio::main]
//! async fn main() {
//!     let (rx, rv) = mpsc::channel::<Message>(32);
//!     let rx_clone = rx.clone();
//!
//!     let handle = tokio::spawn(async move {
//!         let router: Router = RouterBuilder::new()
//!             .send_starting_bytesware(Box::new(SendStartingBytesware))   // Using custom Bytesware, instead of default.
//!             .send_middleware(Box::new(SendMiddleware))
//!             .send_ending_bytesware(Box::new(SendEndingBytesware))
//!             .before(Box::new(DefaultBeforeConnect))                     // Adding funcion that would be ran on connection start
//!             .after(Box::new(DefaultAfterConnect))
//!             .insert("Cool message")                                     // Inserting values into Varmap
//!             .insert(rx_clone)
//!             .build();                                                   // Returning built router 
//! 
//!         router.run().await
//!     });
//! }
//! ```
//! 
//! ### Additional:
//! See [`Varmap`] on how insert is working
//! 
//! [`Bytesware`]: crate::protocol::wares::StartingBytesware
//! [`Middleware`]: crate::protocol::wares::Middleware
//! [`RouterBuilder`]: crate::router::RouterBuilder
//! [`Router`]: crate::router::Router
//! [`Varmap`]: crate::protocol::Varmap

use crate::protocol::wares::Route;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, broadcast};
use std::any::Any;
use std::sync::Arc;
use crate::protocol::Varmap;
use crate::protocol::wares::{BeforeConnect, AfterConnect};
use crate::protocol::wares::{{starting_bytesware, middleware, ending_bytesware}, StartingBytesware, Middleware, EndingBytesware};

mod main_handler;
mod request_handler;
mod send_handler;
mod state;
mod app;

use request_handler::handle_wrapper;
use main_handler::handle_main_thread;
pub use app::App;
pub use state::State;


/// ## RouterBuilder
/// Simple builder patter for [`Router`]. Nothing to say...
/// 
/// *See [`router`] for explanation on how it works*
/// 
/// ## Examples:
/// *copy of [`router`] examples*
///
/// ```
/// use pinguino::router::{Router, RouterBuilder};
/// 
/// #[tokio::main]
/// async fn main() {
///     let (rx, rv) = mpsc::channel::<Message>(32);
///     let rx_clone = rx.clone();
///
///     let handle = tokio::spawn(async move {
///         let router: Router = RouterBuilder::new()
///             .send_starting_bytesware(Box::new(SendStartingBytesware))   // Using custom Bytesware, instead of default.
///             .send_middleware(Box::new(SendMiddleware))
///             .send_ending_bytesware(Box::new(SendEndingBytesware))
///             .before(Box::new(DefaultBeforeConnect))                     // Adding funcion that would be ran on connection start
///             .after(Box::new(DefaultAfterConnect))
///             .insert("Cool message")                                     // Inserting values into Varmap
///             .insert(rx_clone)
///             .build();                                                   // Returning built router 
/// 
///         router.run().await
///     });
/// }
/// ```
/// 
/// [`Router`]: crate::router::Router
/// [`router`]: crate::router
#[derive(Debug)]
pub struct RouterBuilder {  
    pub starting_bytesware: Box<dyn StartingBytesware>,
    pub send: Route,
    pub handshake: Route,
    pub bind: Route,
    pub before: Option<Box<dyn BeforeConnect>>,
    pub after: Option<Box<dyn AfterConnect>>,
    pub varmap: Varmap,
    pub capacity: Option<usize>,                // Capacity of the broadcast channel. If not set will be equal to the 32.
    pub config: Config,
}


/// A TCP router that handles all server logic. It relies on `tokio` `tcp` features and `MPSC`, `broadcast` channels. 
///
/// *its recomended to see [`router`] for explanations on how it works*
/// 
/// ## Examples
/// *This is copy of the [`router`]  examples*
/// ### Basic usage
/// If you want just the built-in version, without any adding any additional features, then you could just do as in the first example.
/// ```
/// use pinguino::router::{Router, RouterBuilder};
/// 
/// #[tokio::main]
/// async fn main(){
///     let router: Router = RouterBuilder::new()
///         .build();
/// 
///     router.run().await
/// }
/// ```
/// 
/// ### Using the [`RouterBuilder`] for router creation.
/// If there is need for custom [`Bytesware`] / [`Middleware`], or some shared resource - it could be done as simple as in the following example.
/// ```
/// use pinguino::router::{Router, RouterBuilder};
/// 
/// #[tokio::main]
/// async fn main() {
///     let handle = tokio::spawn(async move {
///         let router: Router = RouterBuilder::new()
///             .send_starting_bytesware(Box::new(SendStartingBytesware))   // Using custom Bytesware, instead of default.
///             .send_middleware(Box::new(SendMiddleware))
///             .send_ending_bytesware(Box::new(SendEndingBytesware))
///             .before(Box::new(DefaultBeforeConnect))                     // Adding funcion that would be ran on connection start
///             .after(Box::new(DefaultAfterConnect))
///             .insert("Cool message")                                     // Inserting values into Varmap
///             .insert(rx_clone)
///             .build();                                                   // Returning built router 
/// 
///         router.run().await
///     });
/// }
/// ```
/// 
/// ## Panic
/// it **WILL** panic, if it fails to create TcpListener.
/// 
/// [`router`]: crate::router
/// [`Bytesware`]: crate::protocol::wares::StartingBytesware
/// [`Middleware`]: crate::protocol::wares::Middleware
/// [`RouterBuilder`]: crate::router::RouterBuilder
#[derive(Debug)]
pub struct Router {
    pub routes: Arc<Routes>,
    pub before: Option<Box<dyn BeforeConnect>>,
    pub after: Arc<Option<Box<dyn AfterConnect>>>,
    pub extension: Varmap,
    pub capacity: usize,
    pub config: Config,
}


/// ## Routes struct
/// This struct holds [`Route`]'s and is just the way to not have this 3 fields in Router. Thats it. Nothing fancy. See [`Wares`] for more info <3.
/// 
/// [`Route`]: crate::protocol::wares::Route
/// [`Wares`]: crate::protocol::wares
#[derive(Debug)]
pub struct Routes {
    pub starting_bytesware: Box<dyn StartingBytesware>,
    pub send: Route,
    pub handshake: Route,
    pub bind: Route,
}

/// ## RouteRes enum
/// This enum gives information to the Router which [`Method`] had finished working and is it failed or not.
/// For the most cases information about its failing is not important, but if we want to break connection
/// it is important. Yes, I could've made it Result only for `Send`, but it is, what it is.
/// 
/// RouteRes enum is intended to be the output of [`EndingBytesware`]
/// 
/// [`Method`]: crate::protocol::request::Method
/// [`EndingBytesware`]: crate::protocol::wares::ending_bytesware
pub enum RouteRes {
    Handshake(Result<[u8; 512], [u8; 512]>),
    Send(Result<[u8; 512], [u8; 512]>),
    Bind(Result<[u8; 512], [u8; 512]>),
    None(Result<[u8; 512], [u8; 512]>)          // no identified method
}

/// ## Router Config
/// This little thing, provides info to the router on which ip and which port to run. Thats it.
/// Again, this struct exists onl to have less fields in the [`Router`]
/// 
/// ## Example
/// ```
/// let config = Config {
///     ip: "127.0.0.1",
///     port: 3000
/// };
/// ```
/// 
/// [`Router`]: crate::router::Router
#[derive(Debug, Clone)]
pub struct Config {
    pub ip: String,
    pub port: u16,
}

// Well, if you are here anyway, and reading my crappy code...
// This function create listener with given Config. Nothing fancy, works like a charm.
// Yes, it will panic, and yes, providing .expect() message is not that usefull, but still.
async fn create_listener(config: &Config) -> TcpListener {
    let addr = format!("{0}:{1}", config.ip, config.port);
    TcpListener::bind(addr).await.expect("Failed to create listener")
}

/// Simple builder pattern.
/// Fields `send`, `bind`, `handshake`, `config`, `varmap` are not `Option<>` fields,
/// which means that they take their default values at the call of the new() function.
/// 
/// Call of setter *(they are called like that, right?)* functions for mentioned fields will replace default values.
/// Call of setter functions on un mentioned fields will change None to Some(val) 
impl RouterBuilder {
    /// Initiates [`RouterBuilder`]. Important to note, that fields `send`, `bind`, `handshake`, `config`, `varmap` are not `Option<>` fields.
    /// Which means that they take their default values at the call of the new() function.
    /// If you want to squize maximum, you could just manually create [`RouterBuilder`] or [`Router`].
    /// 
    /// [`RouterBuilder`]: crate::router::RouterBuilder
    /// [`Router`]: crate::router::Router
    pub fn new() -> Self {
        // Setting up default StartingBytesware
        let starting_bytesware = Box::new(starting_bytesware::DefaultStartingBytesware);

        
        // Setting up default Send routes.
        let sroute: Route = (
            Box::new(middleware::default_send::DefaultMiddleware),
            Box::new(ending_bytesware::default_send::DefaultEndingBytesware)
        );
    
        // Setting up default Handshake routes.
        let hroute: Route = (
            Box::new(middleware::default_handshake::DefaultMiddleware),
            Box::new(ending_bytesware::default_handshake::DefaultEndingBytesware)
        );
    
        // Setting up default Bind routes.
        let broute: Route = (
            Box::new(middleware::default_bind::DefaultMiddleware),
            Box::new(ending_bytesware::default_bind::DefaultEndingBytesware)
        );

        // Setting up default Config
        let config = Config {
            ip: "127.0.0.1".to_string(),
            port: 8080,
        };

        RouterBuilder {
            starting_bytesware,
            send: sroute,
            handshake: hroute,
            bind: broute,
            varmap: Varmap::new(),
            capacity: None,
            before: None,
            after: None,
            config
        }
    }

    /// Replacing default [`StartingBytesware`] with custom.
    /// 
    /// [`StartingBytesware`]: crate::protocol::wares::starting_bytesware::DefaultStartingBytesware
    pub fn starting_bytesware(mut self, new_val: Box<dyn StartingBytesware>) -> Self {
        self.starting_bytesware = new_val;
        self
    }

    /// Replacing default [`SendMiddleware`] with custom.
    /// 
    /// [`SendMiddleware`]: crate::protocol::wares::middleware::default_send::DefaultMiddleware
    pub fn send_middleware(mut self, new_val: Box<dyn Middleware>) -> Self {
        self.send.0 = new_val;
        self
    }

    /// Replacing default [`SendEndingBytesware`] with custom.
    /// 
    /// [`SendEndingBytesware`]: crate::protocol::wares::ending_bytesware::default_send::DefaultEndingBytesware
    pub fn send_ending_bytesware(mut self, new_val: Box<dyn EndingBytesware>) -> Self {
        self.send.1 = new_val;
        self
    }

    /// Replacing default [`HandshakeMiddleware`] with custom.
    /// 
    /// [`HandshakeMiddleware`]: crate::protocol::wares::middleware::default_handshake::DefaultMiddleware
    pub fn handshake_middleware(mut self, new_val: Box<dyn Middleware>) -> Self {
        self.handshake.0 = new_val;
        self
    }

    /// Replacing default [`HandshakeEndingBytesware`] with custom.
    /// 
    /// [`HandshakeEndingBytesware`]: crate::protocol::wares::ending_bytesware::default_handshake::DefaultEndingBytesware
    pub fn handshake_ending_bytesware(mut self, new_val: Box<dyn EndingBytesware>) -> Self {
        self.handshake.1 = new_val;
        self
    }

    /// Replacing default [`BindMiddleware`] with custom.
    /// 
    /// [`BindMiddleware`]: crate::protocol::wares::middleware::default_bind::DefaultMiddleware
    pub fn bind_middleware(mut self, new_val: Box<dyn Middleware>) -> Self {
        self.bind.0 = new_val;
        self
    }

    /// Replacing default [`BindEndingBytesware`] with custom.
    /// 
    /// [`BindEndingBytesware`]: crate::protocol::wares::ending_bytesware::default_bind::DefaultEndingBytesware
    pub fn bind_ending_bytesware(mut self, new_val: Box<dyn EndingBytesware>) -> Self {
        self.bind.1 = new_val;
        self
    }

    /// Inserting values into [`Router`] to later on be used as shared state in 
    /// custom implementations of the [`StartingBytesware`], [`Middleware`],
    /// [`EndingBytesware`], [`AfterConnection`], [`BeforeConnection`].
    /// 
    /// *See [`Varmap`] for more info on insert()*
    /// 
    /// ## Example
    /// 
    /// ```
    /// let router = RouterBuilder::new()
    ///     .insert("Cool message")
    ///     .build();
    /// ```
    /// 
    /// [`AfterConnection`]: crate::protocol::wares::after_connect
    /// [`BeforeConnection`]:  crate::protocol::wares::before_connect
    pub fn insert<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.varmap.insert(value);
        self
    }

    /// Setting up the capacity of the broadcast channel.
    /// If isnt set, it would be set to the 32
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }
    
    /// Changing default ip to new one.
    /// 
    /// ## Example
    /// 
    /// ```
    /// let router = RouterBuilder::new()
    ///     .ip("0.0.0.0")
    ///     .build();
    /// ```
    pub fn ip(mut self, ip: String) -> Self {
        self.config.ip = ip;
        self
    }

    /// Changing default port to new one.
    /// 
    /// ## Example
    /// 
    /// ```
    /// let router = RouterBuilder::new()
    ///     .port(2222)
    ///     .build();
    /// ```
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Chaning default [`BeforeConnect`] to the custom one
    pub fn before(mut self, before: Box<dyn BeforeConnect>) -> Self {
        self.before = Some(before);
        self
    }

    /// Chaning default [`AfterConnect`] to the custom one
    pub fn after(mut self, after: Box<dyn AfterConnect>) -> Self {
        self.after = Some(after);
        self
    }

    /// Building [`Router`] and setting self.capacity to 32 if not `Some(val)`
    /// 
    /// [`Router`]: crate::router::Router
    pub fn build(self) -> Router {
        let capacity = if let Some(val) = self.capacity {
            val
        } else {
            32
        };

        Router::new(self.starting_bytesware, self.send, self.handshake, self.bind, self.after, self.before, self.varmap, capacity, self.config)
    }
}

impl Router {
    /// If you dont want to use RouterBuilder, or you want to squize maximum startup time (LOL i dont have any arguments)
    /// you could use Router::new() to  
    pub fn new(starting_bytesware: Box<dyn StartingBytesware>, send: Route, handshake: Route, bind: Route, after: Option<Box<dyn AfterConnect>>, before: Option<Box<dyn BeforeConnect>>, extension: Varmap, capacity: usize, config: Config) -> Self {
        Router {
            routes: Arc::new(Routes {
                starting_bytesware,
                send,
                handshake,
                bind,
            }),
            before,
            after: Arc::new(after),
            extension,
            capacity,
            config,
        }
    }

    /// `Router::run()` creates `loop` that recieves incoming request through `TcpLictener::accept()`.
    /// It spawns one additional thread for handling `tokio` channels `MPSC` and `broadcast`. 
    /// First as it recieves connection, it runs [`Before`] (for example - connection incrementer). 
    /// Then it reades bytes `[u8; 512]` from TcpListener.
    /// After that it runs [`StartingBytesware`]. If everything is okay, next [`Middleware`] is executed.
    /// We collect Result<>, no matter of the return we pass it to the [`EndingBytesware`].
    /// When we get the result - RouteRes(Result<[u8; 512], [u8; 512]>) we send it to the client, or
    /// all clients depending on the context. When connection is closed [`After`] is ran (for example - connection decrementer).
    /// 
    /// # Examples
    /// 
    /// ```
    /// let router = RouterBuilder::new().build();
    /// 
    /// router.run().await;
    /// ```
    /// 
    /// # Panic
    /// This function will panic, if it will fail to create listener.
    /// This function may panic with the error "too many opened files" if ulimit is reached (if im not mistaking, because i could be)
    ///
    /// [`Before`]: crate::protocol::wares::before_connect
    /// [`After`]: crate::protocol::wares::after_connect
    pub async fn run(&self) {
        let listener = create_listener(&self.config).await;
        
        println!("Listening to {0}:{1}", self.config.ip, self.config.port);

        let app = Arc::new(Mutex::new(App::new(self.extension.clone())));

        let (mp_tx, mp_rx) = mpsc::unbounded_channel::<[u8; 512]>();
        let (br_tx, _) = broadcast::channel::<[u8; 512]>(self.capacity);

        let main_thread_writer = br_tx.clone();
        tokio::spawn( handle_main_thread(main_thread_writer, mp_rx));

        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(val) => val,
                Err(_) => { continue; }
            };

            let br_tx_sub = br_tx.subscribe();
            let mp_tx_sub = mp_tx.clone();
            let state = Arc::new(Mutex::new(State::new(app.clone(), self.after.clone())));
            
            if let Some(before) = &self.before {
                before.execute(state.clone()).await;
            }
            
            tokio::spawn(handle_wrapper(self.routes.clone(), stream, state, Arc::new(addr), br_tx_sub, mp_tx_sub));
        }
    }
}
