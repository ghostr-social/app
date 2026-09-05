//! A focus jump stops whole-body receipt before replacement playback work.

#[path = "delivery_cancellation_waste_test/assertions.rs"]
mod assertions;
mod delivery_fixture;
mod range_fixture;
#[path = "delivery_cancellation_waste_test/transport.rs"]
mod transport;

use assertions::{assert_cancelled, pending_whole_sequence};
use core::sync::atomic::Ordering;
use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{DataUsageLevel, EngineParams};
use range_fixture::cancellation::BodyKind;
use transport::assert_transport_stops;

const PREFIX: usize = 1_024;
const TOTAL: u64 = 128 * 1_024;

#[tokio::test]
async fn a_focus_jump_stops_whole_body_receipt_before_replacement() {
    let mut old = range_fixture::cancellation::serve(vec![1; PREFIX], TOTAL).await;
    let live = serve_recording("live", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-cancel-waste", serial_options());
    harness.handle.update_focus(focus_now(
        vec![sized_item("old", &old.url, TOTAL, 10_000)],
        0,
        0,
    ));
    let request = tokio::time::timeout(Duration::from_secs(10), &mut old.started)
        .await
        .expect("old request deadline")
        .expect("old request starts");
    assert_eq!(request, BodyKind::Whole);
    assert_eq!(old.bytes_sent.load(Ordering::SeqCst), PREFIX as u64);
    wait_for_ranges(&harness.store, "old", &[(0, PREFIX as u64)]).await;
    let old_sequence = pending_whole_sequence(&harness.handle);
    assert!(harness
        .store
        .present_ranges("live")
        .await
        .expect("live ranges")
        .is_empty());

    harness
        .handle
        .update_focus(focus_now(vec![sized_item("live", &live, 16, 1_000)], 0, 0));
    assert_cancelled(&harness, &old.url, old_sequence).await;
    wait_for_ranges(&harness.store, "live", &[(0, 16)]).await;
    assert_transport_stops(&old, &harness).await;
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
