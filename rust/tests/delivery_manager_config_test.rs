//! A data-usage config change adjusts the transfer concurrency live.

mod range_fixture;
mod support;

use range_fixture::stall::serve_stalling;
use rust_lib_ghostr::engine::{DataUsageLevel, EngineParams};
use support::delivery::start_harness;
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::media_body;
use support::delivery_options::{base_params, DeliveryOptions};
use support::delivery_wait::wait_for_ranges;

#[tokio::test]
async fn delivery_manager_adds_slots_on_higher_data_usage() {
    let first = serve_stalling(media_body()[..4].to_vec(), 16).await;
    let second = serve_stalling(media_body()[..4].to_vec(), 16).await;
    let harness = start_harness("ghostr-delivery-config", capped_options());

    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &first, 16, 1_000),
            sized_item("bb22", &second, 16, 1_000),
        ],
        0,
        0,
    ));

    // Conservative pins one slot: only the current post's chunk runs.
    wait_for_ranges(&harness.store, "aa11", &[(0, 4)]).await;
    let waiting = harness.store.present_ranges("bb22").await.expect("ranges");
    assert!(waiting.is_empty(), "one slot must serialize transfers");

    harness.handle.set_data_usage(DataUsageLevel::Aggressive);

    wait_for_ranges(&harness.store, "bb22", &[(0, 4)]).await;
    std::fs::remove_dir_all(&harness.root).ok();
}

/// Whole-file chunks; one slot when conservative, two when aggressive.
fn capped_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 16,
            conservative_concurrency: 1,
            aggressive_concurrency: 2,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
