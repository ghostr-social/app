mod range_fixture;

use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_delivery::probe::media::probe;
use ghostr_net::transfer_timeouts::TransferTimeouts;

#[tokio::test]
async fn media_probe_falls_back_to_a_one_byte_ranged_get_when_head_is_rejected() {
    let url = range_fixture::reject::serve_head_rejected(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let mut stats = HostStats::new();

    let result = probe(&client, &url, TransferTimeouts::default(), &mut stats)
        .await
        .expect("probe fallback");

    assert_eq!(result.content_length, Some(16));
    assert!(result.accept_ranges);
    assert_eq!(result.content_type.as_deref(), Some("video/mp4"));
    let host = host_of(&url).expect("fixture host");
    assert!(stats.expected_ttfb_ms(&host).is_some());
    assert_eq!(stats.failure_ratio(&host), 0.0);
}
