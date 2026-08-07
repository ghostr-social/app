mod support;

use rust_lib_ghostr::engine::PostId;
use support::delivery::start_harness;
use support::delivery_items::candidate;
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_options::DeliveryOptions;
use support::delivery_wait::wait_for_ranges;

#[tokio::test]
async fn an_admitted_candidate_downloads_without_a_feed_focus_round_trip() {
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-candidate-download", DeliveryOptions::default());

    harness.handle.prioritize_candidate(PostId::new("aa11"));
    harness
        .handle
        .admit_candidate(candidate("aa11", &origin, Some(16), 42));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    assert!(harness.cache.contains("aa11"));
    std::fs::remove_dir_all(&harness.root).ok();
}
