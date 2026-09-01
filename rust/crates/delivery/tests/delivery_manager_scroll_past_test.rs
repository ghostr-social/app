//! Scrolling past a post cancels its in-flight chunk, freeing the
//! transfer slot for the posts still in the window.

mod delivery_fixture;
mod range_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::stats::seed_overall_throughput;
use delivery_fixture::wait::wait_for_ranges;
use delivery_fixture::{start_harness_at, temp_directory};
use ghostr_engine::adaptive::BOOTSTRAP_DIRECT_FETCH_BYTES;
use ghostr_engine::{DataUsageLevel, EngineParams};
use range_fixture::stall::{serve_stalling_signaled, BodyKind};

const RANGE_BYTES: u64 = 64 * 1024;
const STALLED_TOTAL_BYTES: u64 = 2 * BOOTSTRAP_DIRECT_FETCH_BYTES;

#[tokio::test]
async fn delivery_manager_cancels_scrolled_past_transfers() {
    let (stalled, started) =
        serve_stalling_signaled(media_body()[..4].to_vec(), STALLED_TOTAL_BYTES).await;
    let live = serve_recording("live", media_body(), hit_log()).await;
    let root = temp_directory("ghostr-delivery-scroll");
    seed_overall_throughput(&root, RANGE_BYTES);
    let harness = start_harness_at(root, serial_options());

    harness.handle.update_focus(focus_now(
        vec![sized_item("aa11", &stalled, STALLED_TOTAL_BYTES, 1_000)],
        0,
        0,
    ));
    let request = tokio::time::timeout(Duration::from_secs(10), started)
        .await
        .expect("old request deadline")
        .expect("old request starts");
    assert_eq!(request, BodyKind::Range(0..RANGE_BYTES));
    wait_for_ranges(&harness.store, "aa11", &[(0, 4)]).await;

    harness
        .handle
        .update_focus(focus_now(vec![sized_item("bb22", &live, 16, 1_000)], 0, 0));

    // With one slot, bb22 can only complete because aa11 was cancelled.
    wait_for_ranges(&harness.store, "bb22", &[(0, 16)]).await;
    let kept = harness.store.present_ranges("aa11").await.expect("ranges");
    assert_eq!(kept, vec![0..4], "cancelled bytes stay resumable");
    std::fs::remove_dir_all(&harness.root).ok();
}

/// One transfer at a time; the stalled object exceeds WARP's direct-fetch budget.
fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: RANGE_BYTES,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
