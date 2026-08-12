use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub(super) fn is_head(request: &[u8]) -> bool {
    request.starts_with(b"HEAD ")
}

pub(super) async fn write_head(socket: &mut TcpStream, length: usize) {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\
         Accept-Ranges: bytes\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n"
    );
    write(socket, &head).await;
}

pub(super) async fn write_range_headers(socket: &mut TcpStream, length: usize) {
    let head = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-{}/{length}\r\n\
         Content-Length: {length}\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n",
        length - 1
    );
    write(socket, &head).await;
}

async fn write(socket: &mut TcpStream, head: &str) {
    socket
        .write_all(head.as_bytes())
        .await
        .expect("write headers");
    socket.flush().await.expect("flush headers");
}
