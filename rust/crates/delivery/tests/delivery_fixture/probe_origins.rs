//! Probe origins whose successful headers cannot reveal a media length.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn serve_lengthless() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind lengthless probe fixture");
    let address = listener.local_addr().expect("probe fixture address");
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.expect("accept probe");
            tokio::spawn(answer(socket));
        }
    });
    format!("http://{address}/video.mp4")
}

async fn answer(mut socket: TcpStream) {
    let mut request = [0u8; 4096];
    let _ = socket.read(&mut request).await;
    let response = b"HTTP/1.1 200 OK\r\n\
        Content-Type: video/mp4\r\n\
        Accept-Ranges: bytes\r\n\
        Connection: close\r\n\r\n";
    let _ = socket.write_all(response).await;
    let _ = socket.shutdown().await;
}
