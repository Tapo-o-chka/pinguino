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

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^<(?P<key>[a-zA-Z]+)@(?P<value>'[^']+'|\b\w+\b)>$").unwrap());
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Request {
    pub addr: Arc<SocketAddr>,
    pub version: Version,
    pub method: Method,
    pub value: String,
    pub custom: HashMap<String, String>,
    pub varmap: Varmap,
}

#[derive(Debug, Clone)]
pub struct RequestBuilder {
    pub addr: Option<Arc<SocketAddr>>,
    pub version: Option<Version>,
    pub method: Option<Method>,
    pub value: Option<String>,
    pub custom: HashMap<String, String>,
    pub varmap: Varmap,
}

#[allow(unused)]
impl RequestBuilder {
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

    pub fn addr(mut self, addr: Arc<SocketAddr>) -> Self {
        self.addr = Some(addr);
        self
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    pub fn value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    pub fn custom_insert(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }

    pub fn varmap_insert<T: Any + Sync + Send>(mut self, value: T) -> Self {
        self.varmap.insert(value);
        self
    }

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

#[derive(Debug, Clone)]
pub struct RawRequest {
    pub bytes: [u8; 512],
    pub addr: Arc<SocketAddr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    CHAT10,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Bind,
    Handshake,
    Send,
}

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

pub fn extract_version(lines: &mut std::str::Lines<'_>) -> Result<Version, ParseError> {
    let version_line = match lines.next() {
        Some(val) => val,
        None => {
            return Err(ParseError::MissingVersion);
        }
    };
    let version = VERSION_HEADER_RE.captures(version_line)
        .and_then(|cap| cap.name("version").map(|m| m.as_str()));
    
    if let Some(version) = version {
        return match Version::from_str(version) {
            Ok(val) => Ok(val),
            Err(_) => Err(ParseError::MissingVersion)
        };
    } else {
        return Err(ParseError::InvalidFormat);
    }
}

pub fn extract_method(lines: &mut std::str::Lines<'_>) -> Result<Method, ParseError> {
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

    if key == "Method" {
        return match Method::from_str(&value) {
            Ok(val) => Ok(val),
            Err(_) => Err(ParseError::MissingMethod)
        };
    } else {
        return Err(ParseError::MissingMethod);
    }
}

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

#[allow(dead_code)]
impl Request {
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
    
    pub fn from_raw_request(raw_req: RawRequest) -> Result<Self, ParseError> {
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