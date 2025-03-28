use async_trait::async_trait;
use crate::protocol::{request::{ParseError, RawRequest, Request, Version}, response::{Response, ResponseBuilder, ResponseCode}};
use std::fmt::Debug;

#[async_trait]
pub trait StartingBytesware: Debug + Send + Sync {
    async fn bytesware(&self, raw_req: RawRequest) -> Result<Request, Response>;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct DefaultStartingBytesware;

#[async_trait]
impl StartingBytesware for DefaultStartingBytesware {
    async fn bytesware(&self, raw_req: RawRequest) -> Result<Request, Response> {
        let request = Request::from_raw_request(raw_req);
    
        match request {
            Ok(req) => {
                return Ok(req);
            },
            Err(e) => {
                let response = ResponseBuilder::new()
                    .version(Version::CHAT10);

                let code = match e {
                    ParseError::InvalidFormat => ResponseCode::ParseError,
                    ParseError::InvalidKey => ResponseCode::Unauthorized,
                    ParseError::MissingMethod => ResponseCode::InvalidHeader,
                    ParseError::MissingRequestValue => ResponseCode::InvalidName,
                    ParseError::MissingVersion => ResponseCode::InvalidHeader,
                    ParseError::MissingCode => ResponseCode::Error, // How in the world would you get it here?
                    ParseError::NotFound => ResponseCode::Error,
                };

                return Err(response.code(code).build().unwrap());
            }
        }
    }
}