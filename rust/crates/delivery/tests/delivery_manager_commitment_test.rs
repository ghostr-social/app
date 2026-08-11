//! Commitment gives the current item a small serial startup reserve,
//! then yields to ahead startup instead of monopolizing the origin.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{DataUsageLevel, EngineParams};

#[tokio::test]
async fn committed_current_yields_the_serial_slot_before_eof() {
    let current = serve_recording("current", vec![1; 80], hit_log()).await;
    let ahead = serve_recording("ahead", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-delivery-commit", short_head_options());

    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &current, 80, 20_000),
            sized_item("bb22", &ahead, 16, 4_000),
        ],
        0,
        5_000,
    ));

    wait_for_ranges(&harness.store, "aa11", &[(0, 8)]).await;
    wait_for_ranges(&harness.store, "bb22", &[(0, 4)]).await;
    let current_ranges = harness.store.present_ranges("aa11").await.unwrap();
    assert!(current_ranges.iter().all(|range| range.end < 80));
    assert!(!harness.root.join("aa11.video").exists());
    std::fs::remove_dir_all(&harness.root).ok();
}

/// One transfer at a time with four-byte startup/refill ranges.
fn short_head_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            head_seconds: 1,
            chunk_bytes: 4,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
