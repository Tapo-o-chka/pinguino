use socket2::Socket;
use tokio::net::TcpStream;
use std::time::Duration;

/// This function is used to start `Handshake`.
/// I believe there i a better way to do it, but still, it just works.
pub async fn set_keepalive(stream: TcpStream) -> std::io::Result<TcpStream> {
    let std_stream = stream.into_std()?; // Get the standard TcpStream
    let socket = Socket::from(std_stream);

    socket.set_keepalive(true)?; // Enable keepalive
    socket.set_tcp_keepalive(&socket2::TcpKeepalive::new().with_time(Duration::from_secs(60)))?;

    let stream = TcpStream::from_std(socket.into())?; // Convert back to Tokio TcpStream
    Ok(stream)
}
