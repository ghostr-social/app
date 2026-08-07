mod delivery_fixture;
mod raw_http;

use delivery_fixture::media_client;
use ghostr_delivery::probe::media::probe;
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use raw_http::spawn_response_sequence;

#[tokio::test]
async fn media_probe_accepts_a_full_get_when_head_is_rejected() {
    let head = b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
    let get = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 5\r\n\r\nvideo";
    let (url, requests) = spawn_response_sequence(vec![head, get]).await;
    let mut stats = HostStats::new();

    let result = probe(
        &media_client(),
        &url,
        TransferTimeouts::default(),
        &mut stats,
    )
    .await
    .expect("fallback probe");

    assert_eq!(result.content_length, Some(5));
    assert!(!result.accept_ranges);
    assert_eq!(result.content_type.as_deref(), Some("video/mp4"));
    requests.await.expect("upstream requests");
}
