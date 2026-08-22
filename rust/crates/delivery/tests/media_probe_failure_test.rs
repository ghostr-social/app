mod probe_fixture;
mod range_fixture;

use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use probe_fixture::probe;

#[tokio::test]
async fn media_probe_records_a_host_failure_when_every_attempt_is_rejected() {
    let url = range_fixture::reject::serve_failing().await;
    let client = range_fixture::media_client();
    let mut stats = HostStats::new();

    let result = probe(&client, &url, TransferTimeouts::default(), &mut stats).await;

    assert!(result.is_err());
    let host = host_of(&url).expect("fixture host");
    assert!(stats.failure_ratio(&host) > 0.0);
}
