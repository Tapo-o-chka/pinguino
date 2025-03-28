use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::Serialize;

use super::Message;

#[derive(Debug, Row, Serialize)]
pub struct Stat {
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub timestamp: DateTime<Utc>,
    pub ram: u64,
    pub ram_total: u64,
}

impl Stat {
    pub fn from_enum(input: Message) -> Result<Stat, ()>{
        match input {
            Message::Info(val1, val2, val3) => {
                let res = Stat {
                    timestamp: val1,
                    ram: val2,
                    ram_total: val3
                };

                Ok(res)
            },
            Message::Elapsed(_, _, _) => Err(()),
            Message::Error(_, _, _) => Err(())
        }
    }
}