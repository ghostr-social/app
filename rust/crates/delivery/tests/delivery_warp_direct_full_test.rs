//! A reliable tiny-object bound must let WARP acquire the object in one
//! cancellable request instead of paying an eager HEAD round trip.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness_at, temp_directory};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    MediaClass, OriginContext, OriginObservation, OriginQuery, RequestMethod,
};
use std::time::Duration;

#[tokio::test]
async fn reliable_tiny_object_is_fetched_directly_without_head() {
    let log = hit_log();
    let body = media_body();
    let origin = serve_recording("tiny", body.clone(), log.clone()).await;
    let root = temp_directory("warp-direct-full");
    seed_reliable_full_get(&root, &origin, body.len() as u64);
    let harness = start_harness_at(root, DeliveryOptions::default());

    harness.handle.update_focus(focus_now(
        vec![sized_item("tiny", &origin, body.len() as u64, 1_000)],
        0,
        0,
    ));

    let ready = async {
        loop {
            let ranges = harness.store.present_ranges("tiny").await.unwrap();
            if delivery_fixture::wait::covered(&ranges, 0, body.len() as u64) {
                return;
            }
            harness.store.change_notifier().notified().await;
        }
    };
    assert!(
        tokio::time::timeout(Duration::from_secs(2), ready)
            .await
            .is_ok(),
        "selected WARP work did not complete; requests={:?}, plans={:?}, decisions={:?}",
        hits(&log),
        harness.handle.plan_history(),
        harness.handle.decision_history(),
    );
    let requests = hits(&log);
    assert!(requests.iter().any(|hit| hit.starts_with("tiny:GET:")));
    assert!(
        requests.iter().all(|hit| !hit.starts_with("tiny:HEAD:")),
        "direct acquisition must not be preceded by HEAD: {requests:?}"
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

fn seed_reliable_full_get(root: &std::path::Path, url: &str, bytes: u64) {
    let now = unix_time_ms();
    let query = OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::FullGet, bytes, MediaClass::Unknown)
            .with_observed_at_ms(now),
    );
    let mut stats = HostStats::new();
    for _ in 0..4_096 {
        stats.origin_model_mut().observe(
            OriginObservation::success(query.clone(), now)
                .with_ttfb_ms(1)
                .with_throughput_bps(100_000_000),
        );
    }
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("host_stats.json"), stats.to_json()).unwrap();
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
