mod delivery_fixture;

use delivery_fixture::items::candidate;
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::PostId;

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
