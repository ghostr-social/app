use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub(crate) async fn oversized_headers() -> (String, JoinHandle<()>) {
    oversized_status("200 OK").await
}

pub(crate) async fn oversized_status(status: &'static str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("read request") > 0);
        let padding = "a".repeat(33 * 1024);
        let response =
            format!("HTTP/1.1 {status}\r\nX-Padding: {padding}\r\nContent-Length: 0\r\n\r\n");
        socket
            .write_all(response.as_bytes())
            .await
            .expect("response");
    });
    (format!("http://{address}/segment.m4s"), task)
}
