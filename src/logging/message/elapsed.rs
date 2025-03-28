use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::Serialize;
use uuid::Uuid;

use super::Message;

#[derive(Debug, Row, Serialize)]
pub struct Elapsed {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    pub task_id: String,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub timestamp: DateTime<Utc>,
    pub latency: u64,
}

impl Elapsed {
    pub fn from_enum(input: Message) -> Result<Elapsed, ()> {
        match input {
            Message::Elapsed(id, timestamp, latency) => {
                let task_id = id.to_string();
                let latency = latency as u64;
                //println!("Elapsed - Task ID: {:?}, Timestamp: {}, Latency: {}µs", task_id, timestamp, latency);
                return Ok(Elapsed {id: Uuid::new_v4(), task_id, timestamp, latency })
            }
            Message::Error(_, _, _) => {
                Err(())
                
            }
            Message::Info(_, _, _) => {
                Err(())
            }
        }
    }
}