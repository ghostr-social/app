//! A data-usage config change adjusts the transfer concurrency live.

mod delivery_fixture;
mod range_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::media_body;
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{DataUsageLevel, EngineParams};
use range_fixture::stall::serve_stalling;

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
