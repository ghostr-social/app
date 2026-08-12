use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn serve_short_partial() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind short response");
    let address = listener.local_addr().expect("short response address");
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept request");
        let mut request = [0u8; 4096];
        let _ = socket.read(&mut request).await;
        let response = b"HTTP/1.1 206 Partial Content\r\n\
Content-Range: bytes 4-11/16\r\n\
Content-Length: 4\r\n\
Content-Type: video/mp4\r\n\r\n4567";
        socket.write_all(response).await.expect("write response");
    });
    format!("http://{address}/video.mp4")
}
