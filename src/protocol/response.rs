use crate::protocol::request::Version;
use crate::protocol::varmap::Varmap;
use core::str;
use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;
use crate::protocol::request::{extract_version, ParseError, parse_key_value};
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};
/*
    Example:
    ```
        fn main() {
            let bind_request_line = "<CHAT \\ 1.0>\n<Method@Bind>\n<Name@Jeff>";
            let handshake_request_line = "<CHAT \\ 1.0>\n<Method@Handshake>\n<Authorization@0123456789ABCDEF>";
            let send_request_line = "<CHAT \\ 1.0>\n<Method@Send>\n<Message@'Hello world!'>";

            let addr = Arc::new(SocketAddr::from_str("127.0.0.1:3000").expect("Dum dum"));

            let bind_request = Request::parse(bind_request_line, addr.clone());
            let handshake_request = Request::parse(handshake_request_line, addr.clone());
            let send_request = Request::parse(send_request_line, addr.clone());

            println!("1. Bind request: {:?}", bind_request);

            let bind_response = ResponseBuilder::new()
                .version(Version::CHAT10)
                .code(ResponseCode::AuthOK)
                .token("0123456789ABCDEF".to_string())
                .build()
                .unwrap();

            let _ = bind_response.as_bytes().unwrap();
            println!("1. Bind response: {:?}", bind_response.pretty_string());
            
            println!("2. Handshake request: {:?}", handshake_request);
            println!("2. Handshake response: Nothing by design");
            
            println!("3. Send request: {:?}", send_request);

            // Thing that other people are sending :)
            let message_response = ResponseBuilder::new()
                .version(Version::CHAT10)
                .code(ResponseCode::OK)
                .user("Jeff".to_string())
                .message("Hello world, from Jeff!".to_string())
                .build()
                .unwrap();
            let _ = message_response.as_bytes().unwrap(); //This one would be for TcpStream

            println!("3. Message response: {:?}", message_response.pretty_string()); // This function for display purposes, so there is no need to play around with &[u8; 512] and '\0' bytes

            // Custom Response
            let custom_response = ResponseBuilder::new()
                .version(Version::CHAT10)
                .code(ResponseCode::OK)
                .custom_init()
                .custom_insert("Header".to_string(), "'Some value'".to_string());

            let mut custom_headers: HashMap<String, String> = HashMap::new();
            custom_headers.insert("Header1".to_string(), "'Some value 1'".to_string()); // If its a message / value with many words, use '' or client may panic.
            custom_headers.insert("Header2".to_string(), "'Some value 2'".to_string()); // We cant promise order in which headers would be added to the response
            custom_headers.insert("Header3".to_string(), "'Some value 3'".to_string());

            let custom_response_clone = custom_response
                .clone()
                .custom_replace(custom_headers)
                .build()
                .unwrap();

            println!("4. Custom response: {:?}", custom_response.build().unwrap().pretty_string());
            println!("4. Custom response with replace: {:?}", custom_response_clone.pretty_string());
        }
    ```
*/

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Response {
    pub code: ResponseCode,
    pub version: Version,
    pub token: Option<String>,      // <= 32 bytes
    pub user: Option<String>,       // <= 16 bytes
    pub time: Option<DateTime<Utc>>,
    pub message: Option<String>,    // < 512 bytes
    pub custom: Option<HashMap<String, String>>, // Temporary support for custom response building.
    pub varmap: Option<Varmap>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    pub code: Option<ResponseCode>,
    pub version: Option<Version>,
    pub token: Option<String>,      // <= 32 bytes
    pub user: Option<String>,       // <= 16 bytes
    pub time: Option<DateTime<Utc>>,
    pub message: Option<String>,    // < 512 bytes
    pub custom: Option<HashMap<String, String>>, // Temporary support for custom response building. 
    pub varmap: Option<Varmap>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseCode {
    OK,             // <Code@10> (general good)
    AuthOK,         // <Code@11> (binding complete)
    ParseError,     // <Code@20> (no use for now, but reserved for general parse erros)
    InvalidName,    // <Code@21>
    AlreadyTaken,   // <Code@22> (name already taken)
    InvalidHeader,  // <Code@23>
    Unauthorized,   // <Code@24> (Invalid token / token parse failed)
    Error,          // <Code@30> (General error)
    FatalError,     // <Code@31> (Cant recover from this)
    Custom(u8),     // <Code@{val}> 
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BuilderError {
    NoVersion,
    NoCode,
}

#[allow(dead_code)]
impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            code: None,
            version: None,
            token: None,
            user: None,
            time: None,
            message: None,
            custom: None,
            varmap: None,
        }
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn code(mut self, code: ResponseCode) -> Self {
        self.code = Some(code);
        self
    }

    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn time(mut self, time: DateTime<Utc>) -> Self {
        self.time = Some(time);
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn varmap_insert<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.varmap = if let Some(mut varmap) = self.varmap {
            varmap.insert(value);
            Some(varmap)
        } else {
            let mut varmap = Varmap::new();
            varmap.insert(value);
            Some(varmap)
        };
        self
    }

    pub fn custom_init(mut self) -> Self {
        self.custom = Some(HashMap::new());
        self
    }

    pub fn custom_insert(mut self, key: String, value: String) -> Self {
        self.custom.get_or_insert_with(HashMap::new).insert(key, value);
        self
    }

    pub fn custom_replace(mut self, custom: HashMap<String, String>) -> Self {
        self.custom = Some(custom);
        self
    }

    pub fn build(self) -> Result<Response, BuilderError> {
        if let Some(code) = self.code {
            if let Some(version) = self.version {
                return Ok(Response {
                    code: code,
                    version: version,
                    token: self.token,
                    user: self.user,
                    time: self.time,
                    message: self.message,
                    custom: self.custom,
                    varmap: self.varmap,
                });
            }
            return Err(BuilderError::NoVersion);
        }
        // Code and Version must present no matter what.
        Err(BuilderError::NoCode)
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self {
            code: Some(ResponseCode::OK),
            version: Some(Version::CHAT10),
            token: None,
            user: None,
            time: None,
            message: None,
            custom: None,
            varmap: None,
        }
    }
}

#[allow(dead_code)]
impl Response {
    pub fn from_bytes(read_buf: &[u8; 512]) -> Result<Response, ParseError> {
        let response = match str::from_utf8(read_buf) {
            Ok(val) => val.trim_end_matches('\0'),
            Err(_) => {
                return Err(ParseError::InvalidFormat);
            }
        };

        let mut lines = response.lines();

        let version = extract_version(&mut lines)?;
        let code = extract_code(&mut lines)?;

        let mut response = ResponseBuilder::new()
            .version(version)
            .code(code);

        while let Some(line) = lines.next() {
            match extract_user(line) {
                Ok(val) => {
                    response = response.user(val);
                    continue;
                },
                Err(ParseError::NotFound) => {},
                Err(e) => {
                    return Err(e);
                }
            }

            match extract_time(line) {
                Ok(val) => {
                    response = response.time(val);
                    continue;
                },
                Err(ParseError::NotFound) => {},
                Err(e) => {
                    return Err(e);
                }
            }

            match extract_token(line) {
                Ok(val) => {
                    response = response.token(val);
                    continue;
                },
                Err(ParseError::NotFound) => {},
                Err(e) => {
                    return Err(e);
                }
            }

            match extract_message(line) {
                Ok(val) => {
                    response = response.message(val);
                    continue;
                },
                Err(ParseError::NotFound) => {},
                Err(e) => {
                    return Err(e);
                }
            }

            let (key, value) = match parse_key_value(line) {
                Ok(val) => val,
                Err(e) => {
                    return Err(e);
                }
            };

            response = response.custom_insert(key, value);
        }
        
        // Can unwrap, because if we havent found Version or Code yet, we would've exited function already
        Ok(response.build().unwrap())
    }

    pub fn pretty_string(&self) -> String {
        let mut response_line = format!("<CHAT \\ {0}>\n{1}", self.version.to_str(), self.code.to_str());

        if let Some(token) = &self.token {
            response_line += &format!("\n<Token@'{token}'>");
        }

        if let Some(user) = &self.user {
            response_line += &format!("\n<User@'{user}'>");
        }

        if let Some(custom) = &self.custom {
            for (key, value) in custom {
                response_line += &format!("\n<{key}@'{value}'>");
            }
        }

        if let Some(message) = &self.message {
            response_line += &format!("\n<Message@'{message}'>");
        }

        response_line
    }
    
    pub fn as_bytes(&self) -> Result<[u8; 512], ()> {
        let mut response_line = format!("<CHAT \\ {0}>\n{1}", self.version.to_str(), self.code.to_str());

        if let Some(token) = &self.token {
            response_line += &format!("\n<Token@'{token}'>");
        }

        if let Some(user) = &self.user {
            response_line += &format!("\n<User@'{user}'>");
        }

        if let Some(custom) = &self.custom {
            for (key, value) in custom {
                response_line += &format!("\n<{key}@'{value}'>");
            }
        }

        if let Some(message) = &self.message {
            response_line += &format!("\n<Message@'{message}'>");
        }

        if response_line.len() > 512 {
            return Err(());
        }

        let bytes = string_to_bytes(response_line);
        Ok(bytes)
    }
}

impl FromStr for ResponseCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match extract_val(s) {
            Some(10) => Ok(ResponseCode::OK),
            Some(11) => Ok(ResponseCode::AuthOK),
            Some(20) => Ok(ResponseCode::ParseError),
            Some(21) => Ok(ResponseCode::InvalidName),
            Some(22) => Ok(ResponseCode::AlreadyTaken),
            Some(23) => Ok(ResponseCode::InvalidHeader),
            Some(24) => Ok(ResponseCode::Unauthorized),
            Some(30) => Ok(ResponseCode::Error),
            Some(31) => Ok(ResponseCode::FatalError),
            Some(val) => Ok(ResponseCode::Custom(val)),
            None => Err(()),
        }
    }
}

fn extract_code(lines: &mut std::str::Lines<'_>) -> Result<ResponseCode, ParseError>{
    let code_line = match lines.next() {
        Some(val) => val,
        None => {
            return Err(ParseError::MissingCode);
        }
    };

    match ResponseCode::from_str(code_line) {
        Ok(val) => Ok(val),
        Err(_) => Err(ParseError::MissingCode)
    }
}

fn extract_user(line: &str) -> Result<String, ParseError> {
    let (key, value) = match parse_key_value(line) {
        Ok(val) => val,
        Err(e) => {
            println!("key value doesnt like it");
            return Err(e);
        }
    };

    if key == "User" {
        return Ok(value);
    }

    return Err(ParseError::NotFound)
}

fn extract_token(line: &str) -> Result<String, ParseError> {
    let (key, value) = match parse_key_value(line) {
        Ok(val) => val,
        Err(e) => {
            return Err(e);
        }
    };

    if key == "Token" {
        return Ok(value);
    }

    return Err(ParseError::NotFound)
}

fn extract_time(line: &str) -> Result<DateTime<Utc>, ParseError> {
    let (key, value) = match parse_key_value(line) {
        Ok(val) => val,
        Err(e) => {
            return Err(e);
        }
    };

    if key == "Time" {
        let parsed_naive = match NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S") {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::InvalidFormat);
            }
        };
        let parsed_utc: DateTime<Utc> = Utc.from_utc_datetime(&parsed_naive);

        return Ok(parsed_utc);
    }

    return Err(ParseError::NotFound)
}

fn extract_message(line: &str) -> Result<String, ParseError> {
    let (key, value) = match parse_key_value(line) {
        Ok(val) => val,
        Err(e) => {
            return Err(e);
        }
    };

    if key == "Message" {
        return Ok(value);
    }

    return Err(ParseError::NotFound)
}

fn extract_val(input: &str) -> Option<u8> {
    input.strip_prefix("<Code@")?
         .strip_suffix(">")?
         .parse::<u8>()
         .ok()
}

impl ResponseCode {
    pub fn to_str(&self) -> String {
        match self {
            ResponseCode::OK            => "<Code@10>".to_string(),
            ResponseCode::AuthOK        => "<Code@11>".to_string(),
            ResponseCode::ParseError    => "<Code@20>".to_string(),
            ResponseCode::InvalidName   => "<Code@21>".to_string(),
            ResponseCode::AlreadyTaken  => "<Code@22>".to_string(),
            ResponseCode::InvalidHeader => "<Code@23>".to_string(),
            ResponseCode::Unauthorized  => "<Code@24>".to_string(),
            ResponseCode::Error         => "<Code@30>".to_string(),
            ResponseCode::FatalError    => "<Code@31>".to_string(),
            ResponseCode::Custom(val) => format!("<Code@{val}>"),
        }
    }
}

// Utils
pub fn string_to_bytes(input: String) -> [u8; 512] {
    let mut buffer = [0u8; 512]; 
    let bytes = input.as_bytes();
    
    let len = bytes.len().min(512);
    buffer[..len].copy_from_slice(&bytes[..len]);
    
    buffer
}