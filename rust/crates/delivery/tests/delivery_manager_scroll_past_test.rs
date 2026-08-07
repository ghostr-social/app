//! Scrolling past a post cancels its in-flight chunk, freeing the
//! transfer slot for the posts still in the window.

mod delivery_fixture;
mod range_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{DataUsageLevel, EngineParams};
use range_fixture::stall::serve_stalling;

#[tokio::test]
async fn delivery_manager_cancels_scrolled_past_transfers() {
    let stalled = serve_stalling(media_body()[..4].to_vec(), 16).await;
    let live = serve_recording("live", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-delivery-scroll", serial_options());

    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &stalled, 16, 1_000),
            sized_item("bb22", &live, 16, 1_000),
        ],
        0,
        0,
    ));
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

/// One transfer at a time and one whole-file chunk per post.
fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 16,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
