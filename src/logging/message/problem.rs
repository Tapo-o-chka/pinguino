use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::Serialize;
use uuid::Uuid;

use super::{Message, ErrorType};

#[derive(Debug, Row, Serialize)]
pub struct Problem {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    pub task_id: String,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl Problem {
    pub fn from_enum(input: Message) -> Result<Problem, ()> {
        match input {
            Message::Elapsed(_, _, _) => {
                Err(())
            }
            Message::Error(id, timestamp, error) => {
                let task_id = id.to_string();
                //println!("Elapsed - Task ID: {:?}, Timestamp: {}, Latency: {}µs", task_id, timestamp, latency);
                match error {
                    ErrorType::Lagged => {
                        return Ok(Problem {
                            id: Uuid::new_v4(), 
                            task_id, 
                            timestamp, 
                            message: "Lagged".to_string() 
                        });
                    }
                }
            }
            Message::Info(_, _, _) => {
               Err(())
            }
        }
    }
}