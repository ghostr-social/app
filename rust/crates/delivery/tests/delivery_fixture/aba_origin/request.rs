use tokio::io::AsyncReadExt as _;
use tokio::net::TcpStream;

const MAX_HEADER_BYTES: usize = 16 * 1_024;

pub(super) async fn read_line(socket: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(1_024);
    while !bytes.windows(2).any(|window| window == b"\r\n") {
        read_more(socket, &mut bytes).await;
    }
    bytes
}

pub(super) async fn complete_headers(socket: &mut TcpStream, bytes: &mut Vec<u8>) {
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        read_more(socket, bytes).await;
    }
}

async fn read_more(socket: &mut TcpStream, bytes: &mut Vec<u8>) {
    assert!(bytes.len() < MAX_HEADER_BYTES, "request headers too large");
    let mut chunk = [0_u8; 1_024];
    let read = socket.read(&mut chunk).await.expect("request");
    assert!(read > 0, "request ended before headers");
    bytes.extend_from_slice(&chunk[..read]);
}
