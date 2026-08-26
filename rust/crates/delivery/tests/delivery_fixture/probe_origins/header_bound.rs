use super::super::media::HitLog;
use super::note_request;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

pub(super) async fn serve_header_bound_then_complete(log: HitLog, body: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind header-bound fixture");
    let address = listener.local_addr().expect("header-bound address");
    let body = Arc::new(body);
    let full_gets = Arc::new(AtomicUsize::new(0));
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.expect("accept header-bound");
            tokio::spawn(answer(
                socket,
                std::sync::Arc::clone(&log),
                Arc::clone(&body),
                Arc::clone(&full_gets),
            ));
        }
    });
    format!("http://{address}/video.mp4")
}

async fn answer(
    mut socket: TcpStream,
    log: HitLog,
    body: Arc<Vec<u8>>,
    full_gets: Arc<AtomicUsize>,
) {
    let mut request = [0_u8; 4096];
    let read = socket.read(&mut request).await.unwrap_or(0);
    let request = &request[..read];
    note_request(request, &log);
    let text = String::from_utf8_lossy(request);
    let full = text.starts_with("GET ") && !text.to_ascii_lowercase().contains("\r\nrange:");
    if !full {
        write_lengthless_headers(&mut socket).await;
        return;
    }
    let first = full_gets.fetch_add(1, Ordering::SeqCst) == 0;
    write_length_headers(&mut socket, body.len()).await;
    if !first {
        let _ = socket.write_all(&body).await;
    }
    let _ = socket.shutdown().await;
}

async fn write_lengthless_headers(socket: &mut TcpStream) {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n";
    let _ = socket.write_all(response).await;
    let _ = socket.shutdown().await;
}

async fn write_length_headers(socket: &mut TcpStream, length: usize) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n"
    );
    let _ = socket.write_all(response.as_bytes()).await;
}
