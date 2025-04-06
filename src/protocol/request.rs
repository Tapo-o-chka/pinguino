//! ## `Request`
//! 
//! Suprisingly, this module holds all logic needed for handling and operating Requests.
//! Most of the time you would interact with [`Request`] and [`Method`]. And somewhy i hold here [`Version`]
//! which should be shared with [`response`], but let it be.
//! Its recomended to go through examples and some comments on for each struct, but here is
//! a small overview:
//! 
//! The `Request` is what `Router` recieves from `Client`. It should be less then 512 in bytes
//! and should follow the protocol rules in order to be parsed.
//! 
//! ## Example
//! 
//! ### Parsing from the string
//! ```
//! let bind_request_line = "<CHAT \\ 1.0>\n<Method@Bind>\n<Name@Jeff>";
//! let handshake_request_line = "<CHAT \\ 1.0>\n<Method@Handshake>\n<Authorization@'0123456789ABCDEF'>";
//! let send_request_line = "<CHAT \\ 1.0>\n<Method@Send>\n<Message@'Hello world!'>";
//! 
//! let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
//! 
//! let bind_request = Request::parse(bind_request_line, addr.clone());
//! let handshake_request = Request::parse(handshake_request_line, addr.clone());
//! let send_request = Request::parse(send_request_line, addr.clone());
//! 
//! println!("Bind: {:?}", bind_request);
//! println!("Handshake: {:?}", handshake_request);
//! println!("Send: {:?}", send_request);
//! ```
//! 
//! ### Building request
//! ```
//! let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
//! 
//! let request = RequestBuilder::new()
//!     .version(Version::CHAT10)
//!     .method(Method::Send)
//!     .value("Hello world!".to_string())
//!     .addr(addr)
//!     .build()
//!     .unwrap();
//! ```
//! 
//! [`response`]: crate::protocol::response
use regex::Regex;
use once_cell::sync::Lazy;
use core::str;
use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use crate::protocol::varmap::Varmap;

use super::response::string_to_bytes;

// This regex is for general key value extractions
static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^<(?P<key>[a-zA-Z]+)@(?P<value>'[^']+'|\b\w+\b)>$").unwrap());
// This one is special to the Version. Because its not ket value. Obvious, but still want to point it out.
static VERSION_HEADER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^<CHAT \\ (?P<version>[1-9]\.[0-9])>$").unwrap());

/*
    Example:
    ```
        fn main() {
            let bind_request_line = "<CHAT \\ 1.0>\n<Method@Bind>\n<Name@Jeff>";
            let handshake_request_line = "<CHAT \\ 1.0>\n<Method@Handshake>\n<Authorization@0123456789ABCDEF>";
            let send_request_line = "<CHAT \\ 1.0>\n<Method@Send>\n<Message@'Hello world!'>";

            let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());

            let bind_request = Request::parse(bind_request_line, addr.clone());
            let handshake_request = Request::parse(handshake_request_line, addr.clone());
            let send_request = Request::parse(send_request_line, addr.clone());

            println!("Bind: {:?}", bind_request);
            println!("Handshake: {:?}", handshake_request);
            println!("Send: {:?}", send_request);
        }
    ```
*/

/// ## `Request`
/// 
/// This struct is the key part of the `Server` side. Its mostly is interacted with inside of
/// the [`Middleware`]. Request as a string should follow the protocol rules, described in .md file.
/// 
/// Custom headers could be found in `custom` field. It also got [`Varmap`] as a sticky note, cool!
/// No idea why its really needed, because on design level it is working in the environment
/// with its own [`Varmap`] exactly for that purpose.
/// 
/// **ATTENTION** value field behaves differently based on which method is used.
/// 
/// ## Example
/// 
/// ```
/// let request_line = "<CHAT \\ 1.0>\n<Method@Bind>\n<Name@Jeff>";
///
/// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
///
/// let request = Request::parse(request_line, addr).unwrap();
/// 
/// let req_bytes = request.as_bytes().unwrap();
/// 
/// match stream.write(&req_bytes).await {...}
/// ``` 
/// 
/// [`Middleware`]: crate::protocol::wares::middleware
#[derive(Debug, Clone)]
pub struct Request {
    pub addr: Arc<SocketAddr>,
    pub version: Version,
    pub method: Method,
    pub value: String,
    pub custom: HashMap<String, String>,
    pub varmap: Varmap,
}

/// ## `RequestBuilder`
///
/// This struct is mostly designed for the on-client usage, because incoming requests 
/// are parsed via `Request::parse(input, addr)`.
/// 
/// `varmap` and `custom` fields would be copied to the built [`Request`]. Its obvious, 
/// but still want to point it out.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    pub addr: Option<Arc<SocketAddr>>,
    pub version: Option<Version>,
    pub method: Option<Method>,
    pub value: Option<String>,
    pub custom: HashMap<String, String>,
    pub varmap: Varmap,
}

impl RequestBuilder {
    /// Nothing to point out here...
    /// ## Example
    /// 
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn new() -> Self {
        RequestBuilder {
            addr: None,
            version: None,
            method: None,
            value: None,
            custom: HashMap::new(),
            varmap: Varmap::new(),
        }
    }

    /// Setter for `addr` field.
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn addr(mut self, addr: Arc<SocketAddr>) -> Self {
        self.addr = Some(addr);
        self
    }

    /// Setter for `version` field.
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    /// Setter for `method` field.
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    /// Setter for `value` field.
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    /// Inserter for the `custom` field. (custom header field)
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .custom_insert("Header".to_string(), "Value".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn custom_insert(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }

    /// Inserter for the `varmap` field. (sticky note field)
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .varmap_insert("Sticky note")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn varmap_insert<T: Any + Sync + Send>(mut self, value: T) -> Self {
        self.varmap.insert(value);
        self
    }

    /// Just the builder
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<Request, ()> {
        let addr = if let Some(val) = self.addr {
            val
        } else {
            return Err(());
        };

        let version = if let Some(val) = self.version {
            val
        } else {
            return Err(());
        };

        let method = if let Some(val) = self.method {
            val
        } else {
            return Err(());
        };

        let value = if let Some(val) = self.value {
            val
        } else {
            return Err(());
        };

        Ok(Request {
            addr,
            version,
            method,
            value,
            custom: self.custom,
            varmap: self.varmap,
        })
    }
}

/// ## `RawRequest`
/// 
/// This struct is for easier carrying around request bytes and address. 
/// Why do i need to carry around address? Idk, maybe developer would
/// need it for example for the rate limitter, or for location based `Bind`'ing
#[derive(Debug, Clone)]
pub struct RawRequest {
    pub bytes: [u8; 512],
    pub addr: Arc<SocketAddr>,
}

/// ## `Version`
/// 
/// This enum is for general understanding with which version of the protocol
/// we are dealing with. And maybe, based on the version treat request differently.
/// I havent yet figured out how it would be handled diffrently, and in the logic that
/// we have it wont be possible, and if something will be changed in the protocol, it 
/// would most likely will depricieate previous chat implementations, and then why do
/// I even add Version in the first case, but who cares, it wont be used anyway :(
#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    CHAT10,
}

/// ## `Method`
/// 
/// This enum is for general understanding with which method are we dealing with.
/// for now, there are only 3 predefine methods, but there is already thoughts on
/// `Command` method, and custom user-defined methods, but they are still in thoughts.
#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Bind,
    Handshake,
    Send,
}

/// ## `ParseError`
/// 
/// This enum holds info, why parsing broke, and it should be handled. It is 
/// printed with `debug_light` feature on, so you will know, why it did break.
/// 
/// This should be converted to `anyhow` error, or something like that.
#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidFormat,
    InvalidKey,
    MissingVersion,
    MissingMethod,
    MissingRequestValue,
    MissingCode,
    NotFound,
}

/// I believe there is a way, in which this function could be used in application, so i make it public.
/// 
/// This function is used to extract key and value via `Regex` from one line.
/// 
/// ## Example
/// ```
/// let line = "<Key@'Value'";
/// 
/// let (key, value) = parse_key_value(line).unwrap();
/// ```
pub fn parse_key_value(line: &str) -> Result<(String, String), ParseError>{
    let caps = match RE.captures(line) {
        Some(val) => val,
        None => {
            return Err(ParseError::InvalidFormat);
        }
    };

    let key = match caps.name("key") {
        Some(val) => val,
        None => {
            return Err(ParseError::InvalidFormat)
        }
    }.as_str().to_string();
    let value = match caps.name("value") {
        Some(val) => val,
        None => {
            return Err(ParseError::InvalidFormat)
        }
    }.as_str().trim_matches('\'').to_string();

    Ok((key, value))
}

// This function is just version extractor, nothing too fancy here.
pub(crate) fn extract_version(lines: &mut std::str::Lines<'_>) -> Result<Version, ParseError> {
    // Making sure, that there is a line, if no - too sad.
    let version_line = match lines.next() {
        Some(val) => val,
        None => {
            return Err(ParseError::MissingVersion);
        }
    };

    // Using custom regular expersion to retrieve version.
    let version = VERSION_HEADER_RE.captures(version_line)
        .and_then(|cap| cap.name("version").map(|m| m.as_str()));
    
    if let Some(version) = version {
        // If we found it, we need to match it with the `Version` struct.
        return match Version::from_str(version) {
            Ok(val) => Ok(val),
            Err(_) => Err(ParseError::MissingVersion)
        };
    } else {
        return Err(ParseError::InvalidFormat);
    }
}

// Same as in the previous, its just the method extractor. nothing too fancy.
pub(crate) fn extract_method(lines: &mut std::str::Lines<'_>) -> Result<Method, ParseError> {
    // Making sure there is next line
    let line = match lines.next() {
        Some(val) => val,
        None => {
            return Err(ParseError::MissingMethod);
        }
    };
     
    let (key, value) = match parse_key_value(line) {
        Ok(val) => val,
        Err(e) => {
            return Err(e);
        }
    };

    // Need to make sure that the key is `Method`.
    // Why? It will fail anywat to be extracted, if its not the Method?
    // We need to make sure, because there is dumb case, like this:
    // <IdontLikeMethod@Send>
    // And you can do nothing, but check that its method.
    if key == "Method" {
        return match Method::from_str(&value) {
            Ok(val) => Ok(val),
            Err(_) => Err(ParseError::MissingMethod)
        };
    } else {
        return Err(ParseError::MissingMethod);
    }
}

// Just a bigger wrapper around `parse_key_value` function.
fn extract_value(method: &Method, lines: &mut std::str::Lines<'_>) -> Result<String, ParseError> {
    let line = match lines.next() {
        Some(val) => val,
        None => {
            return Err(ParseError::MissingRequestValue)
        }
    };

    let (key, value) = match parse_key_value(line) {
        Ok(val) => val,
        Err(e) => {
            return Err(e);
        }
    };
    let value = value.trim_matches('\'').to_string(); // SHOULD FIX THIS IN REGEX CAPTURING, BUT IM TOO DUMB
    return match method {
        Method::Bind => {
            if key == "Name" {
                Ok(value)
            } else {
                Err(ParseError::InvalidKey)
            }
        },
        Method::Handshake => {
            if key == "Authorization" {
                Ok(value)
            } else {
                Err(ParseError::InvalidKey)
            }
        },
        Method::Send => {
            if key == "Message" {
                Ok(value)
            } else {
                Err(ParseError::InvalidKey)
            }
        }
    };
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bind" => Ok(Method::Bind),
            "Handshake" => Ok(Method::Handshake),
            "Send" => Ok(Method::Send),
            _ => Err(()),
        }
    }
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(Version::CHAT10),
            _ => Err(()),
        }
    }
}

impl Method {
    pub fn to_str(&self) -> &str {
        match self {
            Method::Bind => "Bind",
            Method::Handshake => "Handshake",
            Method::Send => "Send",
        }
    }
}

impl Version {
    pub fn to_str(&self) -> &str {
        match self {
            Version::CHAT10 => "1.0",
        }
    }
}

impl Request {
    /// ## `Request::parse()`
    /// 
    /// This function is desgined to be easiest solution on parsing incoming requests.
    /// It would probably be re-written in `nom` for better efficiency, but right now
    /// its fine.
    /// 
    /// ## Example
    /// ```
    /// let request_line = "<CHAT \\ 1.0>\n<Method@Bind>\n<Name@Jeff>";
    ///
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    ///
    /// let request = Request::parse(request_line, addr).unwrap();
    /// 
    /// println!("Look at my cool incoming request: {:?}", request);
    /// ```
    pub fn parse(input: &str, addr: Arc<SocketAddr>) -> Result<Self, ParseError> {
        let mut lines = input.lines();
    
        let version = extract_version(&mut lines)?;
        let method = extract_method(&mut lines)?;
        let value = extract_value(&method, &mut lines)?;
        let mut custom = HashMap::new();

        while let Some(line) = lines.next() {
            let (key, value) = parse_key_value(line)?;
            custom.insert(key, value);
        }

        Ok(Request {
            addr,
            version,
            method,
            value,
            custom,
            varmap: Varmap::new(),
        })
    }
    
    /// ## `Request::from_raw_request()`
    /// 
    /// Yeeah, `Request::parse()` was designed to be easier solution, but
    /// dragging around `req_bytes` and `addr` was too boring, and i just
    /// stacked those fields inside of `RawRequest`.
    /// 
    /// As `Request::parse()` it should be re-writeen in something like `nom`
    /// for better efficiency, but for now it works great.
    /// 
    /// ## Example
    /// ```
    /// let request_line = "<CHAT \\ 1.0>\n<Method@Bind>\n<Name@Jeff>";
    /// 
    /// // Lets say that we have magic function, that turns String to [u8; 512]
    /// // There is actually function like that in pinguino::protocol::response
    /// let request_bytes = string_to_bytes(request_line.to_string()).unwrap();
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    ///
    /// let raw_req = RawRequest { bytes: request_bytes, addr}
    /// let request = Request::from_raw_request(raw_req).unwrap();
    /// 
    /// println!("Look at my cool incoming request: {:?}", request);
    /// ```
    pub fn from_raw_request(raw_req: RawRequest) -> Result<Self, ParseError> {
        // Same story, as in the Request::parse()
        let line = match str::from_utf8(&raw_req.bytes) {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::InvalidFormat);
            }
        }.trim_end_matches('\0');

        let addr = raw_req.addr;
        let mut lines = line.lines();
    
        let version = extract_version(&mut lines)?;
        let method = extract_method(&mut lines)?;
        let value = extract_value(&method, &mut lines)?;

        let mut custom = HashMap::new();

        while let Some(line) = lines.next() {
            let (key, value) = parse_key_value(line)?;
            custom.insert(key, value);
        }

        Ok(Request {
            addr,
            version,
            method,
            value,
            custom,
            varmap: Varmap::new(),
        })
    }

    /// ## `Request::as_bytes(&self)`
    /// 
    /// This function is designed for `Client` to prepare it for sending.
    /// 
    /// ## Example
    /// ```
    /// let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").unwrap());
    /// let request = RequestBuilder::new()
    ///     .version(Version::CHAT10)
    ///     .method(Method::Send)
    ///     .addr(addr)
    ///     .value("Hello world!".to_string())
    ///     .build()
    ///     .unwrap();
    /// 
    /// let req_bytes = request.as_bytes().unwrap();
    /// 
    /// match stream.write(&req_bytes).await {...}
    /// ```
    pub fn as_bytes(&self) -> Result<[u8; 512], ()> {
        let mut response_line = format!("<CHAT \\ {0}>\n<Method@{1}>", self.version.to_str(), self.method.to_str());

        if self.method == Method::Handshake {
            response_line += format!("\n<Authorization@'{0}'>", self.value).as_str();
        } else if self.method == Method::Bind {
            response_line += format!("\n<Name@'{0}'>", self.value).as_str();
        } else if self.method == Method::Send {
            response_line += format!("\n<Message@'{0}'>", self.value).as_str();
        } else {
            return Err(());
        }

        for (key, value) in &self.custom {
            response_line += format!("\n<{key}@'{value}'").as_str();
        }

        let bytes = string_to_bytes(response_line);
        Ok(bytes)
    }
}