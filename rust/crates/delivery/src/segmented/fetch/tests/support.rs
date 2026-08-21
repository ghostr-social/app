use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub(super) struct LocalClient(Client);

impl MediaHttpRequests for LocalClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub(super) fn client() -> MediaRequestExecutor {
    let client = LocalClient(Client::builder().no_proxy().build().expect("client"));
    MediaRequestExecutor::new(Arc::new(client), MediaRequestLimits::try_new(1, 1).unwrap())
}

pub(super) async fn stalled_body() -> (String, JoinHandle<()>) {
    serve_body(None).await
}

pub(super) async fn stalled_headers() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("read request") > 0);
        std::future::pending::<()>().await;
    });
    (format!("http://{address}/segment.m4s"), task)
}

pub(super) async fn trickled_body(period: Duration) -> (String, JoinHandle<()>) {
    serve_body(Some(period)).await
}

pub(super) async fn oversized_headers() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("read request") > 0);
        let padding = "a".repeat(33 * 1024);
        let response =
            format!("HTTP/1.1 200 OK\r\nX-Padding: {padding}\r\nContent-Length: 1\r\n\r\nx");
        socket
            .write_all(response.as_bytes())
            .await
            .expect("response");
    });
    (format!("http://{address}/segment.m4s"), task)
}

async fn serve_body(period: Option<Duration>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("read request") > 0);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n")
            .await
            .expect("headers");
        if let Some(period) = period {
            for _ in 0..100 {
                tokio::time::sleep(period).await;
                if socket.write_all(b"x").await.is_err() {
                    break;
                }
            }
        } else {
            std::future::pending::<()>().await;
        }
    });
    (format!("http://{address}/segment.m4s"), task)
}
