mod delivery_fixture;

use delivery_fixture::items::candidate;
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_total_len;

#[tokio::test]
async fn an_unsized_provisional_current_retrieves_bytes_without_head() {
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), std::sync::Arc::clone(&log)).await;
    let harness = start_harness("ghostr-candidate-probe", DeliveryOptions::default());

    harness
        .handle
        .admit_candidate(candidate("aa11", &origin, None, 42));

    wait_total_len(&harness.store, "aa11", 16).await;
    let requests = hits(&log);
    assert!(requests.iter().any(|hit| hit.starts_with("origin:GET:")));
    assert!(requests.iter().all(|hit| !hit.contains(":HEAD:")));
    std::fs::remove_dir_all(&harness.root).ok();
}
