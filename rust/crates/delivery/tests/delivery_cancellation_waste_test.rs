//! Once a jump frees the serial slot, a cancelled origin may put at
//! most two transport chunks on the closing socket.

mod delivery_fixture;
mod range_fixture;

use core::sync::atomic::Ordering;
use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{DataUsageLevel, EngineParams};

const PREFIX: usize = 1_024;
const TOTAL: u64 = 128 * 1_024;

#[tokio::test]
async fn a_focus_jump_bounds_bytes_accepted_after_cancellation() {
    let old = range_fixture::cancellation::serve(vec![1; PREFIX], TOTAL).await;
    let live = serve_recording("live", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-cancel-waste", serial_options());
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("old", &old.url, TOTAL, 10_000),
            sized_item("live", &live, 16, 1_000),
        ],
        0,
        0,
    ));
    old.started.await.expect("old request starts");
    wait_for_ranges(&harness.store, "old", &[(0, PREFIX as u64)]).await;

    harness
        .handle
        .update_focus(focus_now(vec![sized_item("live", &live, 16, 1_000)], 0, 0));
    wait_for_ranges(&harness.store, "live", &[(0, 16)]).await;
    old.release.notify_one();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let sent = old.bytes_sent.load(Ordering::SeqCst);
    assert!(sent <= (PREFIX * 3) as u64, "post-cancel bytes: {sent}");
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: TOTAL,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
