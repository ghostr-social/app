use core::ops::Range;
use std::str;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;

mod response;

const MAX_HEADER_BYTES: usize = 16 * 1_024;

pub(super) enum Request {
    Head,
    Body(BodyRequest),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyKind {
    Whole,
    Range,
}

pub(super) enum BodyRequest {
    Whole,
    Range(Range<u64>),
}

impl BodyRequest {
    pub(super) fn kind(&self) -> BodyKind {
        match self {
            Self::Whole => BodyKind::Whole,
            Self::Range(_) => BodyKind::Range,
        }
    }

    pub(super) fn range(&self, total: u64) -> Range<u64> {
        match self {
            Self::Whole => 0..total,
            Self::Range(range) => range.clone(),
        }
    }

    pub(super) async fn write_response(&self, socket: &mut TcpStream, total: u64) {
        let response = match self {
            Self::Whole => response::whole(total),
            Self::Range(range) => response::ranged(range, total),
        };
        socket.write_all(response.as_bytes()).await.ok();
    }
}

pub(super) async fn read(socket: &mut TcpStream, total: u64) -> Request {
    let bytes = read_headers(socket).await;
    let text = str::from_utf8(&bytes).expect("HTTP text");
    if text.starts_with("HEAD ") {
        return Request::Head;
    }
    Request::Body(body_request(text, total))
}

pub(super) async fn write_probe(socket: &mut TcpStream, total: u64) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n\
         Content-Type: video/mp4\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n"
    );
    socket.write_all(response.as_bytes()).await.ok();
}

async fn read_headers(socket: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(1_024);
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        assert!(bytes.len() < MAX_HEADER_BYTES, "request headers too large");
        let mut chunk = [0_u8; 1_024];
        let read = socket.read(&mut chunk).await.expect("request");
        assert!(read > 0, "request ended before headers");
        bytes.extend_from_slice(&chunk[..read]);
    }
    bytes
}

fn body_request(request: &str, total: u64) -> BodyRequest {
    let Some(value) = range_header(request) else {
        return BodyRequest::Whole;
    };
    BodyRequest::Range(parse_range(value, total))
}

fn range_header(request: &str) -> Option<&str> {
    request.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("range").then(|| value.trim())
    })
}

fn parse_range(value: &str, total: u64) -> Range<u64> {
    let (unit, value) = value.split_once('=').expect("range unit");
    assert!(unit.trim().eq_ignore_ascii_case("bytes"), "byte range");
    let (start, end) = value.split_once('-').expect("range values");
    let start = start.parse().expect("range start");
    let end = end.parse::<u64>().unwrap_or(total - 1).min(total - 1);
    start..end + 1
}
