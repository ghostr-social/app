//! Loopback origins that answer with a byte-for-byte canned response, for
//! the malformed headers a real HTTP server would refuse to emit.

#![allow(dead_code)]

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;

pub struct SplitResponse {
    pub url: String,
    pub prefix_sent: oneshot::Receiver<()>,
    pub release: Arc<Notify>,
    pub body_request: oneshot::Receiver<Vec<u8>>,
    pub requests: JoinHandle<()>,
}

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
        assert!(socket.read(&mut buffer).await.expect("read request") > 0);
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

pub async fn spawn_split_response(
    probe: &'static [u8],
    prefix: &'static [u8],
    suffix: &'static [u8],
) -> SplitResponse {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let release = Arc::new(Notify::new());
    let body_release = Arc::clone(&release);
    let (sent, prefix_sent) = oneshot::channel();
    let (request_sent, body_request) = oneshot::channel();
    let requests = tokio::spawn(async move {
        answer_once(&listener, probe).await;
        let (mut socket, _) = listener.accept().await.expect("body request");
        let mut request = vec![0; 4096];
        let length = socket.read(&mut request).await.expect("read body request");
        assert!(length > 0);
        request.truncate(length);
        request_sent.send(request).ok();
        socket.write_all(prefix).await.expect("write prefix");
        sent.send(()).ok();
        body_release.notified().await;
        socket.write_all(suffix).await.expect("write suffix");
    });
    SplitResponse {
        url: format!("http://{address}/video.mp4"),
        prefix_sent,
        release,
        body_request,
        requests,
    }
}

async fn answer_once(listener: &TcpListener, response: &[u8]) {
    let (mut socket, _) = listener.accept().await.expect("request");
    let mut request = vec![0; 4096];
    assert!(socket.read(&mut request).await.expect("read request") > 0);
    socket.write_all(response).await.expect("write response");
}
