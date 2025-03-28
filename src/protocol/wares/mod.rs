pub mod starting_bytesware;
pub mod middleware;
pub mod ending_bytesware;
pub mod before_connect;
pub mod after_connect;

pub use starting_bytesware::StartingBytesware;
pub use middleware::Middleware;
pub use ending_bytesware::EndingBytesware;
pub use before_connect::BeforeConnect;
pub use after_connect::AfterConnect;

pub type Route = (Box<dyn Middleware>, Box<dyn EndingBytesware>);

