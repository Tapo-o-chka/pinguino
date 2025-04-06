//! # Tests for basic auth

use crate::{
    client::{C, DefaultClient},
    router::RouterBuilder
};
use std::{
    net::SocketAddr,
    str::FromStr,
    time::Duration,
};

#[tokio::test]
async fn test_register() {
    tokio::spawn(async move {
        let router = RouterBuilder::new()
            .build();

        router.run().await;
    });

    //Let server start in peace
    tokio::time::sleep(Duration::from_millis(300)).await;

    let target = SocketAddr::from_str("127.0.0.1:8080").unwrap();
    let client = DefaultClient::new(target);

    let _ = match client.bind("Jeff".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            panic!("Failed to BIND the user {:?}", e)
        }
    };
}

#[tokio::test]
async fn test_handshake() {
    tokio::spawn(async move {
        let router = RouterBuilder::new()
            .build();

        router.run().await;
    });

    //Let server start in peace
    tokio::time::sleep(Duration::from_millis(300)).await;

    let target = SocketAddr::from_str("127.0.0.1:8080").unwrap();
    let mut client = DefaultClient::new(target);

    let token = match client.bind("Jeff".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            panic!("Failed to BIND the user {:?}", e)
        }
    };

    match client.handshake(token).await {
        Ok(_) => {},
        Err(e) => {
            panic!("Failed to start handshake {:?}", e)
        }
    }

    match client.send("Hello, hopow your day is going?".to_string()).await {
        Ok(_) => {},
        Err(e) => {
            panic!("Failed to send request {:?}", e)
        }
    }
}