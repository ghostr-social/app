//! A focus update triggers head downloads, current post first.

mod support;

use rust_lib_ghostr::engine::{DataUsageLevel, EngineParams};
use support::delivery::{base_params, start_harness, DeliveryOptions};
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::{hit_log, hits, media_body, serve_recording};
use support::delivery_wait::wait_for_ranges;

#[tokio::test]
async fn delivery_manager_downloads_heads_in_priority_order() {
    let log = hit_log();
    let current = serve_recording("current", media_body(), log.clone()).await;
    let next = serve_recording("next", media_body(), log.clone()).await;
    let harness = start_harness("ghostr-delivery-focus", serial_options());

    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &current, 16, 1_000),
            sized_item("bb22", &next, 16, 1_000),
        ],
        0,
        0,
    ));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    wait_for_ranges(&harness.store, "bb22", &[(0, 16)]).await;
    let recorded = hits(&log);
    assert!(
        recorded.first().is_some_and(|hit| hit.starts_with("current:")),
        "the current post must be fetched first: {recorded:?}"
    );
    assert!(recorded.iter().any(|hit| hit.starts_with("next:")));
    std::fs::remove_dir_all(&harness.root).ok();
}

/// One transfer at a time so the grant order is observable.
fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
