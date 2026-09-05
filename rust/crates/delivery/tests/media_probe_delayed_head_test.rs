mod delivery_fixture;
mod probe_fixture;

use core::time::Duration;
use delivery_fixture::media_client;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use probe_fixture::probe;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::time::Instant;

const HEAD: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";

#[tokio::test]
async fn useful_high_rtt_head_finishes_before_probe_deadline() {
    let (url, request) = delayed_head(Duration::from_millis(450)).await;
    let mut stats = HostStats::new();
    let started = Instant::now();

    let result = probe(
        &media_client(),
        &url,
        TransferTimeouts::default(),
        &mut stats,
    )
    .await
    .expect("450 ms HEAD remains useful");

    assert_eq!(result.content_length, Some(16));
    assert!(started.elapsed() >= Duration::from_millis(400));
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        stats.failure_ratio(&host_of(&url).expect("origin host")),
        0.0
    );
    request.await.expect("origin request");
}

async fn delayed_head(delay: Duration) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let request = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 4_096];
        assert!(
            socket.read(&mut request).await.expect("HEAD request") > 0,
            "origin receives the HEAD request"
        );
        tokio::time::sleep(delay).await;
        socket.write_all(HEAD).await.expect("HEAD response");
    });
    (format!("http://{address}/video.mp4"), request)
}
