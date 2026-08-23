mod probe_fixture;
mod range_fixture;
mod raw_http;

use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use probe_fixture::probe;
use std::time::{SystemTime, UNIX_EPOCH};

const HEAD: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";

#[tokio::test]
async fn head_probe_keeps_final_url_and_network_boundary_time() {
    let (target, target_request) = raw_http::spawn_raw_server(HEAD).await;
    let (start, redirect_request) = raw_http::spawn_redirect(&target).await;
    let mut stats = HostStats::new();
    let before = unix_time_ms();

    let result = probe(
        &range_fixture::media_client(),
        &start,
        TransferTimeouts::default(),
        &mut stats,
    )
    .await
    .expect("redirected HEAD probe");
    let after = unix_time_ms();

    assert_eq!(result.final_url, target);
    assert!((before..=after).contains(&result.observed.observed_at_ms));
    redirect_request.await.unwrap();
    target_request.await.unwrap();
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
