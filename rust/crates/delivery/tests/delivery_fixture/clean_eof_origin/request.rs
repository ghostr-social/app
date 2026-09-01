use tokio::io::AsyncReadExt as _;
use tokio::net::TcpStream;

const MAX_HEADER_BYTES: usize = 16 * 1_024;

pub(super) enum Request {
    Head,
    Range,
    Whole,
}

pub(super) async fn read(socket: &mut TcpStream) -> Request {
    let bytes = read_headers(socket).await;
    let text = str::from_utf8(&bytes).expect("HTTP text");
    if text.starts_with("HEAD ") {
        return Request::Head;
    }
    if has_range(text) {
        return Request::Range;
    }
    Request::Whole
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

fn has_range(request: &str) -> bool {
    request.lines().any(|line| {
        line.split_once(':')
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case("range"))
    })
}
