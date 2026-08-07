//! Loopback origins that answer with a byte-for-byte canned response, for
//! the truncated and malformed bodies a real HTTP server would not emit.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub async fn spawn_raw_server(response: &'static [u8]) -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let request = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut buffer = vec![0; 4096];
        let bytes = socket.read(&mut buffer).await.expect("read request");
        socket.write_all(response).await.expect("write response");
        buffer.truncate(bytes);
        buffer
    });
    (format!("http://{address}/video.mp4"), request)
}

/// An address nothing is listening on, so a connection is refused rather
/// than answered.
pub async fn unused_loopback_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    drop(listener);
    format!("http://{address}/video.mp4")
}
