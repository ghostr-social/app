//! A ranged origin whose tail is released only after the test proves
//! delivery gave the serial slot to a replacement post.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{ops::Range, str};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Notify};

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
    let (mut socket, _) = input.listener.accept().await.expect("accept");
    let range = read_request(&mut socket, input.total).await;
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

async fn read_request(socket: &mut TcpStream, total: u64) -> Range<u64> {
    let mut request = vec![0u8; 4_096];
    let read = socket.read(&mut request).await.expect("request");
    let request = str::from_utf8(&request[..read]).expect("HTTP text");
    let value = request
        .lines()
        .find_map(|line| line.strip_prefix("range: bytes="))
        .expect("Range header");
    let (start, end) = value.split_once('-').expect("range values");
    let start = start.parse().expect("range start");
    let end = end.parse::<u64>().unwrap_or(total - 1).min(total - 1);
    start..end + 1
}

async fn write_head(socket: &mut TcpStream, range: &Range<u64>, total: u64) {
    let head = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {}-{}/{total}\r\n\
         Content-Length: {}\r\nContent-Type: video/mp4\r\n\r\n",
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
