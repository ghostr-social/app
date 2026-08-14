mod delivery_fixture;
mod raw_http;

use delivery_fixture::media_client;
use ghostr_delivery::probe::media::probe;
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use raw_http::spawn_response_sequence;

#[tokio::test]
async fn media_probe_stops_after_a_rejected_head() {
    let head = b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
    let (url, requests) = spawn_response_sequence(vec![head]).await;
    let mut stats = HostStats::new();

    let result = probe(
        &media_client(),
        &url,
        TransferTimeouts::default(),
        &mut stats,
    )
    .await;

    assert!(result.is_err());
    requests.await.expect("upstream requests");
}
