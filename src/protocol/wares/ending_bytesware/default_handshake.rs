use async_trait::async_trait;
use crate::protocol::response::Response;
use crate::protocol::wares::EndingBytesware;

#[derive(Debug)]
pub struct DefaultEndingBytesware;

#[async_trait]
impl EndingBytesware for DefaultEndingBytesware {
    async fn bytesware(&self, res: Result<Response, Response>) -> Result<[u8; 512], [u8; 512]> {
        match res {
            Ok(res) => {
                return Ok(res.as_bytes().unwrap())
            },
            Err(res) => {
                return Err(res.as_bytes().unwrap())
            }
        }
    }
}