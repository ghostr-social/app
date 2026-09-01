//! A body origin whose tail is released only after the test proves
//! delivery gave the serial slot to a replacement post.

use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt as _;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Notify};

mod request;
pub use request::BodyKind;

const TAIL_CHUNK: usize = 1_024;

pub struct CancellableOrigin {
    pub url: String,
    pub bytes_sent: Arc<AtomicU64>,
    pub release: Arc<Notify>,
    pub finished: Arc<Notify>,
    pub started: oneshot::Receiver<BodyKind>,
}

struct StreamInput {
    listener: TcpListener,
    prefix: Vec<u8>,
    total: u64,
    sent: Arc<AtomicU64>,
    release: Arc<Notify>,
    finished: Arc<Notify>,
    started: oneshot::Sender<BodyKind>,
}

pub async fn serve(prefix: Vec<u8>, total: u64) -> CancellableOrigin {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    let bytes_sent = Arc::new(AtomicU64::new(0));
    let release = Arc::new(Notify::new());
    let finished = Arc::new(Notify::new());
    let (started, observed) = oneshot::channel();
    tokio::spawn(stream_once(StreamInput {
        listener,
        prefix,
        total,
        sent: Arc::clone(&bytes_sent),
        release: Arc::clone(&release),
        finished: Arc::clone(&finished),
        started,
    }));
    CancellableOrigin {
        url: format!("http://{address}/video.mp4"),
        bytes_sent,
        release,
        finished,
        started: observed,
    }
}

async fn stream_once(input: StreamInput) {
    let (mut socket, body) = accept_body_request(&input.listener, input.total).await;
    let range = body.range(input.total);
    body.write_response(&mut socket, input.total).await;
    let range_bytes = range.end - range.start;
    let prefix_len = input.prefix.len().min(range_bytes as usize);
    write_body(&mut socket, &input.prefix[..prefix_len], &input.sent).await;
    input.started.send(body.kind()).ok();
    input.release.notified().await;
    let tail = vec![9; TAIL_CHUNK];
    while input.sent.load(Ordering::SeqCst) < range_bytes {
        if !write_body(&mut socket, &tail, &input.sent).await {
            break;
        }
        tokio::time::sleep(core::time::Duration::from_millis(5)).await;
    }
    input.finished.notify_one();
}

async fn accept_body_request(
    listener: &TcpListener,
    total: u64,
) -> (TcpStream, request::BodyRequest) {
    loop {
        let (mut socket, _) = listener.accept().await.expect("accept");
        match request::read(&mut socket, total).await {
            request::Request::Head => request::write_probe(&mut socket, total).await,
            request::Request::Body(body) => return (socket, body),
        }
    }
}

async fn write_body(socket: &mut TcpStream, bytes: &[u8], sent: &AtomicU64) -> bool {
    if socket.write_all(bytes).await.is_err() || socket.flush().await.is_err() {
        return false;
    }
    sent.fetch_add(bytes.len() as u64, Ordering::SeqCst);
    true
}
