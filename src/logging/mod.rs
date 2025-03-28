use std::{sync::atomic::{AtomicUsize, Ordering}, time::Duration};
use tokio::time::{sleep, Instant};
use chrono::Utc;
use tokio::sync::mpsc::{Receiver, Sender};
use clickhouse::Client;
use sysinfo::{Pid, System};

mod message;

pub use message::{Message, Elapsed, Problem, Stat, ErrorType, Connect};

static TOTAL_CONNECTED: AtomicUsize = AtomicUsize::new(0);

pub fn increment_total_connected() {
    TOTAL_CONNECTED.fetch_add(1, Ordering::SeqCst);
}

pub fn decrement_total_connected() {
    TOTAL_CONNECTED.fetch_sub(1, Ordering::SeqCst);
}

pub fn get_total_connected() -> usize {
    TOTAL_CONNECTED.load(Ordering::SeqCst)
}

pub async fn main(mut receiver: Receiver<Message>, db: DbConfig) {
    tokio::spawn(async move {

        let mut client = Client::default()
            .with_url(db.url);

        if let Some(database) = db.database {
            client = client.with_database(database);
        }

        if let Some(username) = db.username {
            client = client.with_user(username);
        }

        if let Some(password) = db.password {
            client = client.with_password(password);
        }

        client.query(
            "CREATE TABLE IF NOT EXISTS Elapsed (
                id UUID,
                task_id String,
                timestamp DateTime,
                latency UInt64
            ) ENGINE = MergeTree ORDER BY id"
        ).execute().await.unwrap();

        client.query(
            "CREATE TABLE IF NOT EXISTS Problem (
                id UUID,
                task_id String,
                timestamp DateTime,
                message String
            ) ENGINE = MergeTree ORDER BY id"
        ).execute().await.unwrap();

        client.query(
            "CREATE TABLE IF NOT EXISTS Stats (
                timestamp DateTime,
                ram UInt64,
                ram_total UInt64
            ) ENGINE = MergeTree ORDER BY timestamp"
        ).execute().await.unwrap();

        client.query(
            "CREATE TABLE IF NOT EXISTS Connect (
                total UInt64,
                timestamp DateTime
            ) Engine = MergeTree ORDER BY timestamp"
        ).execute().await.unwrap();
    
        let mut inserter_1 = client.inserter("Elapsed")
            .unwrap()
            .with_timeouts(Some(Duration::from_secs(5)), Some(Duration::from_secs(20)))
            .with_max_bytes(50_000_000)
            .with_max_rows(750_000);
        let mut inserter_2 = client.inserter("Problem")
            .unwrap()
            .with_timeouts(Some(Duration::from_secs(5)), Some(Duration::from_secs(20)))
            .with_max_bytes(50_000_000)
            .with_max_rows(750_000);
        let mut inserter_3 = client.inserter("Stats")
            .unwrap()
            .with_timeouts(Some(Duration::from_secs(5)), Some(Duration::from_secs(20)))
            .with_max_bytes(50_000_000)
            .with_max_rows(750_000);
        let mut inserter_4 = client.inserter("Connect")
            .unwrap()
            .with_timeouts(Some(Duration::from_secs(5)), Some(Duration::from_secs(20)))
            .with_max_bytes(50_000_000)
            .with_max_rows(750_000);        

        let mut timer = Instant::now();
        while let Some(message) = receiver.recv().await {
            match message {
                Message::Elapsed(_, _, _) => {
                    let elapsed = Elapsed::from_enum(message).unwrap();
                    inserter_1.write(&elapsed).unwrap();
                },
                Message::Error(_, _, _) => {
                    let problem = Problem::from_enum(message).unwrap();
                    inserter_2.write(&problem).unwrap();
                },
                Message::Info(_, _, _) => {
                    let stat = Stat::from_enum(message).unwrap();
                    inserter_3.write(&stat).unwrap();
                }
            }

            let connected = Connect { total: get_total_connected() as u64, timestamp: Utc::now() };
            inserter_4.write(&connected).unwrap();

            if timer.elapsed() > Duration::from_secs(2) {
                let _ = inserter_1.force_commit().await.unwrap();
                let _ = inserter_2.force_commit().await.unwrap();
                let _ = inserter_3.force_commit().await.unwrap();
                let _ = inserter_4.force_commit().await.unwrap();

                timer = Instant::now();
            }
        }

        inserter_1.end().await.unwrap();
        inserter_2.end().await.unwrap();
        inserter_3.end().await.unwrap();
    });
}

pub async fn sysmterics(rx: Sender<Message>) {
    let mut system = System::new_all();
    let pid = std::process::id(); // Get current process ID

    loop {
        system.refresh_all(); // Refresh system info
        if let Some(process) = system.process(Pid::from_u32(pid)) {
            let message = Message::Info(Utc::now(), process.memory() / (1024 * 1024), system.used_memory() / (1024 * 1024));
            match rx.send(message).await {
                Ok(_) => {},
                Err(_) => todo!(),
            }
        } else {
            println!("Process not found!");
        }
        sleep(Duration::from_secs(5)).await;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DbConfig {
    url: String,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl Default for DbConfig {
    fn default() -> Self {
        DbConfig { 
            url: "http://127.0.0.1:8123".to_string(), 
            database: None,
            username: Some("tapock".to_string()), 
            password: Some("password".to_string())
        }
    }
}
