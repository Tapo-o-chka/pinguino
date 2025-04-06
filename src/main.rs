mod example;
mod protocol;
mod app;
mod client;
mod tests;
mod logging;
mod router;

use crate::router::{Router, RouterBuilder};
use crate::app::send::{SendStartingBytesware, SendMiddleware, SendEndingBytesware};
use std::time::Duration;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::time;
use client::Client;
use logging::Message;
use crate::protocol::wares::{before_connect::DefaultBeforeConnect, after_connect::DefaultAfterConnect};
use tokio::sync::mpsc;


#[tokio::main]
async fn main() {
    let (rx, _) = mpsc::channel::<Message>(32);
    let rx_clone = rx.clone();
    tokio::spawn(async move {
        let router: Router = RouterBuilder::new()
            .starting_bytesware(Box::new(SendStartingBytesware))
            .send_middleware(Box::new(SendMiddleware))
            .send_ending_bytesware(Box::new(SendEndingBytesware))
            .before(Box::new(DefaultBeforeConnect))
            .after(Box::new(DefaultAfterConnect))
            .insert("Cool message")
            .insert(rx_clone)
            .build();

        println!("Starting...");
        router.run().await;
        println!("Ended!");
    });

    let client = Client::default();

    time::sleep(Duration::from_secs(3)).await;
    println!("Running!");
    client.bind("Jeffry".to_string()).await.unwrap();
    client.handshake().await.unwrap();

    client.send("Hello world!".to_string()).await.unwrap();

    let sub = client.subscribe().await;

    tokio::spawn(async move {
        let mut reciever = sub.lock().await;

        while let Some(message) = reciever.recv().await {
            println!("Got message:\n{0}", message.pretty_string());
        }
    });
    
    loop {
        let mut stdin = BufReader::new(stdin());
        let mut line = String::new();

        match stdin.read_line(&mut line).await {
            Ok(0) => {
                println!("EOF reached or no input provided");
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to read line: {}", e);
            }
        }
        if line.strip_suffix('\n').unwrap() == "quit" {
            client.terminate().await.unwrap();
            println!("Terminated");
            break;
        }
        
        client.send(line).await.unwrap();
    }
}
