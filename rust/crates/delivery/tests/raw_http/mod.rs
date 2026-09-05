//! Loopback origins that answer with a byte-for-byte canned response, for
//! the malformed headers a real HTTP server would refuse to emit.

#![expect(dead_code, reason = "shared fixture APIs vary by integration scenario")]

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

mod body_request;
mod gated_response;
mod split_response;

#[expect(
    unused_imports,
    reason = "shared fixture APIs vary by integration scenario"
)]
pub use gated_response::{spawn_gated_response, GatedResponse};
#[expect(
    unused_imports,
    reason = "shared fixture APIs vary by integration scenario"
)]
pub use split_response::{spawn_gated_split_response, spawn_split_response, SplitResponse};

pub struct StalledHeaders {
    pub url: String,
    pub request_started: oneshot::Receiver<()>,
    pub requests: JoinHandle<()>,
}

pub async fn spawn_stalled_headers() -> StalledHeaders {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let (started, request_started) = oneshot::channel();
    let requests = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut buffer = vec![0; 4096];
        assert!(
            socket.read(&mut buffer).await.expect("read request") > 0,
            "the client must start its request"
        );
        started.send(()).ok();
        let _ = socket.read(&mut buffer[..1]).await;
    });
    StalledHeaders {
        url: format!("http://{address}/video.mp4"),
        request_started,
        requests,
    }
}

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

pub async fn spawn_redirect(target: &str) -> (String, JoinHandle<Vec<u8>>) {
    let response = format!(
        "HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let request = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut buffer = vec![0; 4096];
        let bytes = socket.read(&mut buffer).await.expect("read request");
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write redirect");
        buffer.truncate(bytes);
        buffer
    });
    (format!("http://{address}/video.mp4"), request)
}

pub async fn spawn_response_sequence(responses: Vec<&'static [u8]>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let requests = tokio::spawn(async move {
        for response in responses {
            let (mut socket, _) = listener.accept().await.expect("request");
            let mut buffer = vec![0; 4096];
            let bytes = socket.read(&mut buffer).await.expect("read request");
            assert!(bytes > 0, "empty request");
            socket.write_all(response).await.expect("write response");
        }
    });
    (format!("http://{address}/video.mp4"), requests)
}
