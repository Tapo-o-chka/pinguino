use clickhouse::Row;
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Row, Serialize)]
pub struct Connect {
    pub total: u64,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub timestamp: DateTime<Utc>,
}
