use super::answer_once;
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;

pub struct GatedResponse {
    pub url: String,
    pub release_headers: Arc<Notify>,
    pub body_request: oneshot::Receiver<Vec<u8>>,
    pub requests: JoinHandle<()>,
}

pub async fn spawn_gated_response(probe: &'static [u8], body: &'static [u8]) -> GatedResponse {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let release_headers = Arc::new(Notify::new());
    let release = Arc::clone(&release_headers);
    let (request_sent, body_request) = oneshot::channel();
    let requests = tokio::spawn(async move {
        answer_once(&listener, probe).await;
        let (mut socket, _) = listener.accept().await.expect("body request");
        let mut request = vec![0; 4096];
        let length = socket.read(&mut request).await.expect("read body request");
        request.truncate(length);
        request_sent.send(request).ok();
        release.notified().await;
        socket.write_all(body).await.expect("write response");
    });
    GatedResponse {
        url: format!("http://{address}/video.mp4"),
        release_headers,
        body_request,
        requests,
    }
}
