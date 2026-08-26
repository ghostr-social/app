mod delivery_fixture;
mod probe_fixture;

use delivery_fixture::media_client;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::response_limits::MAX_MEDIA_RESPONSE_HEADER_BYTES;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use probe_fixture::probe;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[tokio::test]
async fn media_probe_rejects_response_headers_above_the_media_limit() {
    let (url, request) = oversized_head_origin().await;
    let mut stats = HostStats::new();

    let requests = media_client();
    let error = probe(&requests, &url, TransferTimeouts::default(), &mut stats)
        .await
        .expect_err("oversized HEAD headers must be rejected");
    let request = String::from_utf8(request.await.expect("origin request")).expect("HTTP request");

    assert!(error.to_string().contains("headers exceed byte limit"));
    assert!(request.starts_with("HEAD "));
    assert!(stats.failure_ratio(&host_of(&url).expect("origin host")) > 0.0);
}

async fn oversized_head_origin() -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("origin address");
    let padding = "a".repeat(MAX_MEDIA_RESPONSE_HEADER_BYTES);
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nX-Padding: {padding}\r\n\r\n"
    );
    let request = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = vec![0; 4096];
        let length = socket.read(&mut request).await.expect("read request");
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write response");
        request.truncate(length);
        request
    });
    (format!("http://{address}/video.mp4"), request)
}
