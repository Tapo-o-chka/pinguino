use socket2::Socket;
use tokio::net::TcpStream;
use std::time::Duration;
use clap::Parser;

pub async fn set_keepalive(stream: TcpStream) -> std::io::Result<TcpStream> {
    let std_stream = stream.into_std()?; // Get the standard TcpStream
    let socket = Socket::from(std_stream);

    socket.set_keepalive(true)?; // Enable keepalive
    socket.set_tcp_keepalive(&socket2::TcpKeepalive::new().with_time(Duration::from_secs(60)))?;

    let stream = TcpStream::from_std(socket.into())?; // Convert back to Tokio TcpStream
    Ok(stream)
}

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,
    #[arg(short, long, default_value = "8080")]
    pub port: String,
    #[arg(short, long, default_value = "1")]
    pub mode: String,
}
