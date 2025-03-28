pub mod elapsed;
pub mod problem;
pub mod stat;
pub mod connect;

use tokio::task::Id;
use chrono::{DateTime, Utc};

pub use elapsed::Elapsed;
pub use problem::Problem;
pub use stat::Stat;
pub use connect::Connect;

pub enum Message {
    Elapsed(Id, DateTime<Utc>, u128),        // TaskId, Time stamp, Latency in micros.
    Error(Id, DateTime<Utc>, ErrorType),     // TaskId, Time stamp, Type of the error.
    Info(DateTime<Utc>, u64, u64),                // Time stamp, how many new clients
}

pub enum ErrorType {
    Lagged
}
