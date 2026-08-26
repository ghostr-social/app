use core::time::Duration;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub(crate) async fn immediate_asset() -> (String, JoinHandle<()>) {
    serve(Duration::ZERO, "200 OK").await
}

pub(crate) async fn immediate_failure() -> (String, JoinHandle<()>) {
    serve(Duration::ZERO, "500 Internal Server Error").await
}

pub(crate) async fn immediate_status(status: &'static str) -> (String, JoinHandle<()>) {
    serve(Duration::ZERO, status).await
}

pub(crate) async fn delayed_asset(delay: Duration) -> (String, JoinHandle<()>) {
    serve(delay, "200 OK").await
}

async fn serve(delay: Duration, status: &'static str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("read request") > 0);
        tokio::time::sleep(delay).await;
        let response =
            format!("HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: 1\r\n\r\nx");
        socket
            .write_all(response.as_bytes())
            .await
            .expect("response");
    });
    (format!("http://{address}/segment.m4s"), task)
}
