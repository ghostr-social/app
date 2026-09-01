use core::ops::Range;
use tokio::io::AsyncWriteExt as _;
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

pub(super) async fn write_body_headers(
    socket: &mut TcpStream,
    request: &[u8],
    length: usize,
) -> Range<usize> {
    let range = requested_range(request, length);
    let head = range
        .as_ref()
        .map_or_else(|| whole(length), |range| ranged(range, length));
    write(socket, &head).await;
    range.unwrap_or(0..length)
}

fn whole(length: usize) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\
         Accept-Ranges: bytes\r\nContent-Type: video/mp4\r\nETag: \"fixture-aba\"\r\n\
         Connection: close\r\n\r\n"
    )
}

fn ranged(range: &Range<usize>, length: usize) -> String {
    let head = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {}-{}/{length}\r\n\
         Content-Length: {}\r\nContent-Type: video/mp4\r\nETag: \"fixture-aba\"\r\n\
         Connection: close\r\n\r\n",
        range.start,
        range.end - 1,
        range.end - range.start
    );
    head
}

fn requested_range(request: &[u8], length: usize) -> Option<Range<usize>> {
    let request = core::str::from_utf8(request).expect("HTTP text");
    let value = request.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("range").then(|| value.trim())
    })?;
    let (unit, bounds) = value.split_once('=').expect("range unit");
    assert!(unit.eq_ignore_ascii_case("bytes"), "byte range");
    let (start, end) = bounds.split_once('-').expect("range values");
    let start = start.parse().expect("range start");
    let end = end.parse::<usize>().unwrap_or(length - 1).min(length - 1);
    assert!(start <= end, "non-empty range");
    Some(start..end + 1)
}

async fn write(socket: &mut TcpStream, head: &str) {
    socket
        .write_all(head.as_bytes())
        .await
        .expect("write headers");
    socket.flush().await.expect("flush headers");
}
