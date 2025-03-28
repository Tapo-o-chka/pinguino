# **Pinguino**
(pinguino - few grammatic mistakes, it should be like this, trust me)

## What is this crate about
The useless TCP chat implementation, with no one asked custom protocol. All messages have fixed size of `[u8; 512]` including headers. This crate highly relies on `tokio` and its pretty light wrapper around what tokio provides. For keepalive Im using `socket2`.

**Important**: if you want to actually load test, and you somewhy get unexpectadly low results, try to change `ulimit` to higher than default value.

**IMPORTANT**: This crate is not ready for usage

## Why is it called that
As the penguin, `pinguino` is a bird with wings, but wont fly even if you would force it to flap its wings very fast.

## Example
The goal was to keep things simple, and still flexible.
### Minimum working example
**Server**
```rs
use pinguino::protocol::router::{Router, RouterBuilder};

#[tokio::main]
async fn main() {
    let router: Router = RouterBuilder::new()
        .build();

    router.run().await
}
```

**Client**
```rs
use std::net::SocketAddr;
use pinguino::client

#[tokio::main]
async fn main() {
    let target = SocketAddr::from_str("127.0.0.1:8080").unwrap();

    let mut client = DefaultClient::new(target);

    // Bind user and get the token
    let token = client.bind("Jeffry".to_string()).await.unwrap();
    client.handshake(token).await.unwrap();

    // Connect via handshake
    client.handshake(token).await.unwrap();

    // You can manually send messages out of loop / subscribtion!
    client.send("Hello world!".to_string()).await.unwrap();

    // Subscribe to messages (recieve)
    let sub = client.subscribe().await;

    // Spawn listener thread
    tokio::spawn(async move {
        let mut reciever = sub.lock().await;

        while let Some(message) = reciever.recv().await {
            println!("Got message:\n{0}", message.pretty_string());
        }
    });

    // Writing loop
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
```

## Features
- `["debug_light"]` - adding built-in debug messages (via println!()) for errors and when connection is started / closed.
- `["debug_full"]` - adding additional info on messages that are sent and recieved via tokio MPSC / broadcast / TcpStream / TcpListener on top of what `["debug_light"]` provides.

You can use neither, or one of them. There is no point of including both of them, because `["debug_full"]` includes everything that `["debug_light"]` provides.

## How this crate is intended to be used
~~*It doesnt, but still.*~~ Developers can use this crate as some sort of *framework* to create custom 

## Current achievements (What am I proud of)
### Latest load test:
On Ryzen 5800x cpu I managed to get up to 900 concurent clients without errors / lost clients, and up to 1240 clients without being Lagged by tokio MPSC / broadcast channels. *it is 1240 senders * 1240 recievers every 1200 ms*

## Goals
- Scalable messages
- Move [u8; 512] to Bytes with capacity 512
- Add custom rooms
- Add built-in tools (such as rate limiting)

## Questions
- Lib provided tracing for errors (in addition to Debug modes) is needed? What info is needed?
- Is there need for custom request parser written with `nom` for example, instead of regex?
