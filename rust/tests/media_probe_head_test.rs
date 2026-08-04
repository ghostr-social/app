mod range_fixture;

use rust_lib_ghostr::engine::host_stats::{host_of, HostStats};
use rust_lib_ghostr::video::media_probe::probe;
use rust_lib_ghostr::video::transfer_timeouts::TransferTimeouts;

#[tokio::test]
async fn media_probe_head_learns_length_range_support_and_content_type() {
    let url = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let mut stats = HostStats::new();

    let result = probe(&client, &url, TransferTimeouts::default(), &mut stats)
        .await
        .expect("probe");

    assert_eq!(result.content_length, Some(16));
    assert!(result.accept_ranges);
    assert_eq!(result.content_type.as_deref(), Some("video/mp4"));
    let host = host_of(&url).expect("fixture host");
    assert!(stats.expected_ttfb_ms(&host).is_some());
    assert_eq!(stats.failure_ratio(&host), 0.0);
}
