//! A fragmented Range header must not masquerade as a whole-body request.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::clean_eof_origin::{serve, BODY};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;

#[tokio::test]
async fn fragmented_range_request_does_not_signal_whole_body_recovery() {
    let mut origin = serve().await;
    let mut socket = TcpStream::connect(address(origin.url())).await.unwrap();
    write_fragmented_range(&mut socket).await;

    let signalled =
        tokio::time::timeout(Duration::from_millis(50), origin.wait_whole_started()).await;

    assert!(signalled.is_err(), "range request was classified as whole");
    assert!(read_response(&mut socket).await.ends_with(BODY));
}

fn address(url: &str) -> (String, u16) {
    let parsed = url::Url::parse(url).expect("valid test fixture");
    let host = parsed.host_str().expect("valid test fixture").to_owned();
    let port = parsed.port().expect("valid test fixture");
    (host, port)
}

async fn write_fragmented_range(socket: &mut TcpStream) {
    let mut first = b"GET /video.mp4 HTTP/1.1\r\nHost: localhost\r\nX-Pad: ".to_vec();
    first.resize(first.len() + 4_096, b'a');
    first.extend_from_slice(b"\r\n");
    socket.write_all(&first).await.unwrap();
    socket
        .write_all(b"range: bytes=0-7\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
}

async fn read_response(socket: &mut TcpStream) -> Vec<u8> {
    let mut response = Vec::new();
    tokio::time::timeout(Duration::from_secs(1), socket.read_to_end(&mut response))
        .await
        .expect("range response completes")
        .expect("valid test fixture");
    response
}
