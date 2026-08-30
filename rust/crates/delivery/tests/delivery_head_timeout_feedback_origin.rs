//! Stalled HEAD origin with a start signal for feedback timing assertions.

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const BODY: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n0123456789abcdef";

pub struct Origin {
    pub url: String,
    pub head_started: oneshot::Receiver<()>,
    pub requests: JoinHandle<Requests>,
}

pub struct Requests {
    pub head: Vec<u8>,
    pub body: Vec<u8>,
}

pub async fn serve() -> Origin {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let (started, head_started) = oneshot::channel();
    let requests = tokio::spawn(async move {
        let (mut stalled, _) = listener.accept().await.expect("HEAD request");
        let head = read_headers(&mut stalled).await;
        started.send(()).ok();
        let _ = stalled.read(&mut [0]).await;
        let (mut playable, _) = listener.accept().await.expect("body request");
        let body = read_headers(&mut playable).await;
        playable.write_all(BODY).await.expect("body response");
        Requests { head, body }
    });
    Origin {
        url: format!("http://{address}/video.mp4"),
        head_started,
        requests,
    }
}

async fn read_headers(socket: &mut TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
        let mut buffer = [0; 1_024];
        let read = socket.read(&mut buffer).await.expect("request headers");
        assert!(read > 0, "request ended before headers");
        request.extend_from_slice(&buffer[..read]);
    }
    request
}
