//! Raw TCP fixture that answers one ranged request with a 206 header
//! for `total` bytes, sends only `prefix`, then stalls forever.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

pub async fn serve_stalling(prefix: Vec<u8>, total: u64) -> String {
    serve_stalling_signaled(prefix, total).await.0
}

pub async fn serve_stalling_signaled(
    prefix: Vec<u8>,
    total: u64,
) -> (String, oneshot::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind stall fixture");
    let address = listener.local_addr().expect("stall address");
    let (started, observed) = oneshot::channel();
    tokio::spawn(async move {
        let mut started = Some(started);
        loop {
            let (mut socket, _) = listener.accept().await.expect("stall accept");
            let mut request = vec![0u8; 4096];
            let read = socket.read(&mut request).await.unwrap_or(0);
            if request[..read].starts_with(b"HEAD ") {
                write_head(&mut socket, total).await;
                continue;
            }
            started.take().expect("first range").send(()).ok();
            stall(&mut socket, &prefix, total).await;
            return;
        }
    });
    (format!("http://{address}/video.mp4"), observed)
}

async fn stall(socket: &mut TcpStream, prefix: &[u8], total: u64) {
    let head = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-{}/{total}\r\n\
         Content-Length: {total}\r\nContent-Type: video/mp4\r\n\
         ETag: \"fixture-stall\"\r\n\r\n",
        total - 1
    );
    let _ = socket.write_all(head.as_bytes()).await;
    let _ = socket.write_all(prefix).await;
    let _ = socket.flush().await;
    std::future::pending::<()>().await
}

async fn write_head(socket: &mut TcpStream, total: u64) {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n\
         Accept-Ranges: bytes\r\nContent-Type: video/mp4\r\n\
         ETag: \"fixture-stall\"\r\n\r\n"
    );
    socket.write_all(head.as_bytes()).await.ok();
}
