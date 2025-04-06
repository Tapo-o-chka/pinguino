//! ## Protocol
//! 
//! This module is focused on handling main objects, that user will interact with.
//! For example - [`Request`] and [`Response`] are objects that developer will interact with
//! when he will be working on custom client, or [`wares`].
//! 
//! This module also holds [`Varmap`].
//! 
//! [`Request`]: crate::protocol::request::Request
//! [`Response`]: crate::protocol::response::Response
//! [`wares`]: crate::protocol::wares

pub mod request;
pub mod response;
mod utils;
mod varmap;
pub mod wares;

pub use varmap::Varmap;
pub(crate) use utils::set_keepalive;