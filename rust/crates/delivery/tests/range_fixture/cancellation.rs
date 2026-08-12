//! A ranged origin whose tail is released only after the test proves
//! delivery gave the serial slot to a replacement post.

use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Notify};

mod request;

const TAIL_CHUNK: usize = 1_024;

pub struct CancellableOrigin {
    pub url: String,
    pub bytes_sent: Arc<AtomicU64>,
    pub release: Arc<Notify>,
    pub started: oneshot::Receiver<()>,
}

struct StreamInput {
    listener: TcpListener,
    prefix: Vec<u8>,
    total: u64,
    sent: Arc<AtomicU64>,
    release: Arc<Notify>,
    started: oneshot::Sender<()>,
}

pub async fn serve(prefix: Vec<u8>, total: u64) -> CancellableOrigin {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    let bytes_sent = Arc::new(AtomicU64::new(0));
    let release = Arc::new(Notify::new());
    let (started, observed) = oneshot::channel();
    tokio::spawn(stream_once(StreamInput {
        listener,
        prefix,
        total,
        sent: Arc::clone(&bytes_sent),
        release: Arc::clone(&release),
        started,
    }));
    CancellableOrigin {
        url: format!("http://{address}/video.mp4"),
        bytes_sent,
        release,
        started: observed,
    }
}

async fn stream_once(input: StreamInput) {
    let (mut socket, range) = accept_range_request(&input.listener, input.total).await;
    write_head(&mut socket, &range, input.total).await;
    let range_bytes = range.end - range.start;
    let prefix_len = input.prefix.len().min(range_bytes as usize);
    write_body(&mut socket, &input.prefix[..prefix_len], &input.sent).await;
    input.started.send(()).ok();
    input.release.notified().await;
    let tail = vec![9; TAIL_CHUNK];
    while input.sent.load(Ordering::SeqCst) < range_bytes {
        if !write_body(&mut socket, &tail, &input.sent).await {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

async fn accept_range_request(listener: &TcpListener, total: u64) -> (TcpStream, Range<u64>) {
    loop {
        let (mut socket, _) = listener.accept().await.expect("accept");
        match request::read(&mut socket, total).await {
            request::Request::Head => request::write_probe(&mut socket, total).await,
            request::Request::Range(range) => return (socket, range),
        }
    }
}

async fn write_head(socket: &mut TcpStream, range: &Range<u64>, total: u64) {
    let head = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {}-{}/{total}\r\n\
         Content-Length: {}\r\nContent-Type: video/mp4\r\nAccept-Ranges: bytes\r\n\r\n",
        range.start,
        range.end - 1,
        range.end - range.start
    );
    socket.write_all(head.as_bytes()).await.ok();
}

async fn write_body(socket: &mut TcpStream, bytes: &[u8], sent: &AtomicU64) -> bool {
    if socket.write_all(bytes).await.is_err() || socket.flush().await.is_err() {
        return false;
    }
    sent.fetch_add(bytes.len() as u64, Ordering::SeqCst);
    true
}
