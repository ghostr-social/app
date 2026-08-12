use std::ops::Range;
use std::str;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct PartialFailureOrigin {
    url: String,
    starts: Arc<Mutex<Vec<u64>>>,
}

pub async fn serve(total: u64) -> PartialFailureOrigin {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind partial");
    let address = listener.local_addr().expect("partial address");
    let starts = Arc::new(Mutex::new(Vec::new()));
    tokio::spawn(accept(listener, total, starts.clone()));
    PartialFailureOrigin {
        url: format!("http://{address}/video.mp4"),
        starts,
    }
}

impl PartialFailureOrigin {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn attempts(&self) -> usize {
        self.starts.lock().expect("starts").len()
    }

    pub fn starts(&self) -> Vec<u64> {
        self.starts.lock().expect("starts").clone()
    }
}

async fn accept(listener: TcpListener, total: u64, starts: Arc<Mutex<Vec<u64>>>) {
    while let Ok((socket, _)) = listener.accept().await {
        let starts = starts.clone();
        tokio::spawn(async move { answer(socket, total, starts).await });
    }
}

async fn answer(mut socket: TcpStream, total: u64, starts: Arc<Mutex<Vec<u64>>>) {
    let range = read_range(&mut socket, total).await;
    starts.lock().expect("starts").push(range.start);
    let length = range.end - range.start;
    let head = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {}-{}/{total}\r\n\
         Content-Length: {length}\r\nContent-Type: video/mp4\r\n\r\n",
        range.start,
        range.end - 1,
    );
    socket.write_all(head.as_bytes()).await.ok();
    let partial = (length / 2).max(1).min(length.saturating_sub(1));
    socket.write_all(&vec![7; partial as usize]).await.ok();
}

async fn read_range(socket: &mut TcpStream, total: u64) -> Range<u64> {
    let mut request = [0; 4096];
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
