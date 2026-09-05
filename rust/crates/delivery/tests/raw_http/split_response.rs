use super::body_request::accept_body;
use std::sync::Arc;
use tokio::io::AsyncWriteExt as _;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;

pub struct SplitResponse {
    pub url: String,
    pub prefix_sent: oneshot::Receiver<()>,
    pub release_headers: Arc<Notify>,
    pub release: Arc<Notify>,
    pub body_request: oneshot::Receiver<Vec<u8>>,
    pub requests: JoinHandle<()>,
}

pub async fn spawn_split_response(
    probe: &'static [u8],
    prefix: &'static [u8],
    suffix: &'static [u8],
) -> SplitResponse {
    spawn(probe, prefix, suffix, false).await
}

pub async fn spawn_gated_split_response(
    probe: &'static [u8],
    prefix: &'static [u8],
    suffix: &'static [u8],
) -> SplitResponse {
    spawn(probe, prefix, suffix, true).await
}

async fn spawn(
    probe: &'static [u8],
    prefix: &'static [u8],
    suffix: &'static [u8],
    gated: bool,
) -> SplitResponse {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let release_headers = Arc::new(Notify::new());
    let header_release = Arc::clone(&release_headers);
    if !gated {
        release_headers.notify_one();
    }
    let release = Arc::new(Notify::new());
    let body_release = Arc::clone(&release);
    let (sent, prefix_sent) = oneshot::channel();
    let (request_sent, body_request) = oneshot::channel();
    let requests = tokio::spawn(async move {
        let (mut socket, request) = accept_body(&listener, probe).await;
        request_sent.send(request).ok();
        header_release.notified().await;
        socket.write_all(prefix).await.expect("write prefix");
        sent.send(()).ok();
        body_release.notified().await;
        socket.write_all(suffix).await.expect("write suffix");
    });
    SplitResponse {
        url: format!("http://{address}/video.mp4"),
        prefix_sent,
        release_headers,
        release,
        body_request,
        requests,
    }
}
