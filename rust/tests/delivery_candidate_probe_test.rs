mod support;

use support::delivery::start_harness;
use support::delivery_items::candidate;
use support::delivery_media::{hit_log, hits, media_body, serve_recording};
use support::delivery_options::DeliveryOptions;
use support::delivery_wait::wait_total_len;

#[tokio::test]
async fn an_unsized_candidate_enters_the_probe_pool_before_focus() {
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), log.clone()).await;
    let harness = start_harness("ghostr-candidate-probe", DeliveryOptions::default());

    harness
        .handle
        .admit_candidate(candidate("aa11", &origin, None, 42));

    wait_total_len(&harness.store, "aa11", 16).await;
    assert!(hits(&log).iter().any(|hit| hit.starts_with("origin:HEAD:")));
    std::fs::remove_dir_all(&harness.root).ok();
}
