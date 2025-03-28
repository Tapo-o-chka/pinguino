mod example;
mod protocol;
mod app;
mod client;
mod tests;
mod logging;
mod router;

use crate::router::{Router, RouterBuilder};
use crate::app::send::{SendStartingBytesware, SendMiddleware, SendEndingBytesware};

use logging::Message;
use crate::protocol::wares::{before_connect::DefaultBeforeConnect, after_connect::DefaultAfterConnect};
use tokio::sync::mpsc;


#[tokio::main]
async fn main() {
    let (rx, rv) = mpsc::channel::<Message>(32);
    let rx_clone = rx.clone();
    let handle = tokio::spawn(async move {
        let router: Router = RouterBuilder::new()
            .starting_bytesware(Box::new(SendStartingBytesware))
            .send_middleware(Box::new(SendMiddleware))
            .send_ending_bytesware(Box::new(SendEndingBytesware))
            .before(Box::new(DefaultBeforeConnect))
            .after(Box::new(DefaultAfterConnect))
            .insert("Cool message")
            .insert(rx_clone)
            .build();

        router.run().await
    });
    let _ = logging::main(rv, logging::DbConfig::default()).await;
    let _ = logging::sysmterics(rx).await;
    let _ = handle.await;

    /*
    let target = match std::net::SocketAddr::from_str("127.0.0.1:8080") {
        Ok(val) => val,
        Err(_) => { return; }
    };

    let mut client = DefaultClient::new(target);

    time::sleep(Duration::from_secs(3)).await;
    println!("Running!");
    let token = client.bind("Jeffry".to_string()).await.unwrap();
    client.handshake(token).await.unwrap();

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
    */
}
