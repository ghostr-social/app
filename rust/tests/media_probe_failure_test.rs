mod range_fixture;

use rust_lib_ghostr::engine::host_stats::{host_of, HostStats};
use rust_lib_ghostr::video::media_probe::probe;
use rust_lib_ghostr::video::transfer_timeouts::TransferTimeouts;

#[tokio::test]
async fn media_probe_records_a_host_failure_when_every_attempt_is_rejected() {
    let url = range_fixture::reject::serve_failing().await;
    let client = range_fixture::media_client();
    let mut stats = HostStats::new();

    let result = probe(&client, &url, TransferTimeouts::default(), &mut stats).await;

    assert!(result.is_err());
    let host = host_of(&url).expect("fixture host");
    assert!(stats.failure_ratio(&host) > 0.0);
    assert!(stats.expected_ttfb_ms(&host).is_none());
}
