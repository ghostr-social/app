//! Origin fixture whose HEAD redirects to an occupied authority while GET is direct.

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const BODY: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n0123456789abcdef";

pub struct Origin {
    pub url: String,
    pub blocked_url: String,
    pub head_started: oneshot::Receiver<()>,
    pub requests: JoinHandle<Requests>,
    _blocked: TcpListener,
}

pub struct Requests {
    pub head: Vec<u8>,
    pub body: Vec<u8>,
}

pub async fn serve() -> Origin {
    let blocked = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("blocked origin");
    let blocked_url = format!(
        "http://{}/video.mp4",
        blocked.local_addr().expect("address")
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("origin");
    let address = listener.local_addr().expect("address");
    let redirect = redirect_to(&blocked_url);
    let (started, head_started) = oneshot::channel();
    let requests = tokio::spawn(async move {
        let (mut head_socket, _) = listener.accept().await.expect("HEAD request");
        let head = read_headers(&mut head_socket).await;
        started.send(()).ok();
        head_socket.write_all(&redirect).await.expect("redirect");
        let (mut body_socket, _) = listener.accept().await.expect("body request");
        let body = read_headers(&mut body_socket).await;
        body_socket.write_all(BODY).await.expect("body response");
        Requests { head, body }
    });
    Origin {
        url: format!("http://{address}/video.mp4"),
        blocked_url,
        head_started,
        requests,
        _blocked: blocked,
    }
}

fn redirect_to(target: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .into_bytes()
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
