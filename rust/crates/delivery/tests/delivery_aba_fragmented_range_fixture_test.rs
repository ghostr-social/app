//! The controlled ABA origin classifies a request only after all headers arrive.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::aba_origin::serve;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;

#[tokio::test]
async fn fragmented_range_header_still_receives_an_exact_partial_response() {
    let (url, origin) = serve(b"abcdefgh".to_vec()).await;
    let address = url
        .strip_prefix("http://")
        .and_then(|value| value.strip_suffix("/video.mp4"))
        .expect("fixture address");
    let mut socket = TcpStream::connect(address).await.expect("connect fixture");
    socket
        .write_all(b"GET /video.mp4 HTTP/1.1\r\nHost: fixture\r\nRan")
        .await
        .expect("first header fragment");
    socket.flush().await.expect("flush first fragment");
    origin.wait_for_hits(1).await;
    origin.release_first_headers();
    socket
        .write_all(b"ge: bytes=2-3\r\nConnection: close\r\n\r\n")
        .await
        .expect("second header fragment");

    let response = tokio::time::timeout(Duration::from_secs(10), read_headers(&mut socket))
        .await
        .expect("response header deadline");
    assert!(response.starts_with("HTTP/1.1 206 Partial Content\r\n"));
    assert!(response.contains("Content-Range: bytes 2-3/8\r\n"));
    assert!(response.contains("Content-Length: 2\r\n"));
}

async fn read_headers(socket: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        let mut chunk = [0_u8; 512];
        let read = socket.read(&mut chunk).await.expect("response");
        assert!(read > 0, "response ended before headers");
        bytes.extend_from_slice(&chunk[..read]);
    }
    String::from_utf8(bytes).expect("HTTP text")
}
