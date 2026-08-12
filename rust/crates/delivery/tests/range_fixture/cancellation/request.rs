use std::ops::Range;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub(super) enum Request {
    Head,
    Range(Range<u64>),
}

pub(super) async fn read(socket: &mut TcpStream, total: u64) -> Request {
    let mut bytes = [0_u8; 4_096];
    let read = socket.read(&mut bytes).await.expect("request");
    let text = str::from_utf8(&bytes[..read]).expect("HTTP text");
    if text.starts_with("HEAD ") {
        return Request::Head;
    }
    Request::Range(parse_range(text, total))
}

pub(super) async fn write_probe(socket: &mut TcpStream, total: u64) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n\
         Content-Type: video/mp4\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n"
    );
    socket.write_all(response.as_bytes()).await.ok();
}

fn parse_range(request: &str, total: u64) -> Range<u64> {
    let value = request
        .lines()
        .find_map(|line| line.strip_prefix("range: bytes="))
        .expect("Range header");
    let (start, end) = value.split_once('-').expect("range values");
    let start = start.parse().expect("range start");
    let end = end.parse::<u64>().unwrap_or(total - 1).min(total - 1);
    start..end + 1
}
