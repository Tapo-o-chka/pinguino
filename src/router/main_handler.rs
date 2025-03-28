use tokio::sync::{mpsc::UnboundedReceiver, broadcast::Sender};

/// Well, this function just transfers messages from senders to listeners... and thats it...
pub async fn handle_main_thread(main_thread_writer: Sender<[u8; 512]>, mut mp_rx: UnboundedReceiver<[u8; 512]>) {
    while let Some(message) = mp_rx.recv().await {
        match main_thread_writer.send(message) {
            Ok(_val) => {
                #[cfg(feature = "debug_full")]
                println!("<0> [MAIN] Sent {_val} bytes to broadcast");
            },
            Err(_e) => {
                #[cfg(feature = "debug_full")]
                println!(">0< [MAIN] Failed to send message to broadcast with error {_e}");
            },
        }
    }
}
