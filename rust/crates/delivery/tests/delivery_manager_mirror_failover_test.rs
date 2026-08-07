//! A post whose first source is broken switches to the healthy mirror
//! in its source set instead of hammering the broken one.

mod delivery_fixture;

use delivery_fixture::items::focus_now;
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording, serve_rejecting};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn delivery_manager_falls_back_to_a_healthy_mirror() {
    let log = hit_log();
    let broken = serve_rejecting("broken", log.clone()).await;
    let mirror = serve_recording("mirror", media_body(), log.clone()).await;
    let harness = start_harness("ghostr-delivery-mirror", DeliveryOptions::default());

    harness
        .handle
        .update_focus(focus_now(vec![mirrored("aa11", &broken, &mirror)], 0, 0));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    let attempts = hits(&log)
        .iter()
        .filter(|hit| hit.starts_with("broken:"))
        .count();
    assert!(
        attempts <= 3,
        "the broken source must be abandoned, not retried: {:?}",
        hits(&log)
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

/// One post advertising two sources, the broken one first.
fn mirrored(id: &'static str, first: &str, second: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![first.to_owned(), second.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
    }
}
