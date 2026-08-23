//! Probe origins whose successful headers cannot reveal a media length.

use super::media::HitLog;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod header_bound;

pub const RANGE_BLIND_BODY: &[u8] = b"0123456789abcdef";

pub async fn serve_lengthless() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind lengthless probe fixture");
    let address = listener.local_addr().expect("probe fixture address");
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.expect("accept probe");
            tokio::spawn(answer(socket));
        }
    });
    format!("http://{address}/video.mp4")
}

pub async fn serve_recording_range_blind(log: HitLog) -> String {
    serve_recording_range_blind_body(log, RANGE_BLIND_BODY.to_vec()).await
}

pub async fn serve_recording_range_blind_body(log: HitLog, body: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind range-blind probe fixture");
    let address = listener.local_addr().expect("probe fixture address");
    let body = Arc::new(body);
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.expect("accept probe");
            tokio::spawn(answer_recording(socket, log.clone(), Arc::clone(&body)));
        }
    });
    format!("http://{address}/video.mp4")
}

pub async fn serve_header_bound_then_complete(log: HitLog, body: Vec<u8>) -> String {
    header_bound::serve_header_bound_then_complete(log, body).await
}

async fn answer(mut socket: TcpStream) {
    let mut request = [0u8; 4096];
    let _ = socket.read(&mut request).await;
    let response = b"HTTP/1.1 200 OK\r\n\
        Content-Type: video/mp4\r\n\
        Accept-Ranges: bytes\r\n\
        Connection: close\r\n\r\n";
    let _ = socket.write_all(response).await;
    let _ = socket.shutdown().await;
}

async fn answer_recording(mut socket: TcpStream, log: HitLog, body: Arc<Vec<u8>>) {
    let mut request = [0u8; 4096];
    let read = socket.read(&mut request).await.unwrap_or(0);
    note_request(&request[..read], &log);
    let response = b"HTTP/1.1 200 OK\r\n\
        Content-Type: video/mp4\r\n\
        Connection: close\r\n\r\n";
    let _ = socket.write_all(response).await;
    let _ = socket.write_all(&body).await;
    let _ = socket.shutdown().await;
}

fn note_request(request: &[u8], log: &HitLog) {
    let text = String::from_utf8_lossy(request);
    let method = text.split_whitespace().next().unwrap_or("unknown");
    let range = text
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with("range:"))
        .map(str::trim)
        .unwrap_or("full");
    log.lock()
        .expect("hit log")
        .push(format!("{method}:{range}"));
}
