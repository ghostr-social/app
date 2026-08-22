mod probe_fixture;
mod range_fixture;

use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use probe_fixture::probe;

#[tokio::test]
async fn rejected_head_does_not_issue_an_unplanned_body_request() {
    let url = range_fixture::reject::serve_head_rejected(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let mut stats = HostStats::new();

    let result = probe(&client, &url, TransferTimeouts::default(), &mut stats).await;

    assert!(result.is_err());
    let host = host_of(&url).expect("fixture host");
    assert!(stats.failure_ratio(&host) > 0.0);
}
