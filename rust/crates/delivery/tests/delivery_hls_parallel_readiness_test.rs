mod delivery_fixture;
mod hls_terminal_wait;

use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::hls_start::wait_for_starts;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn focused_hls_videos_prepare_in_parallel_and_require_bootstrap_assets() {
    let gate = HlsGate::new();
    let first = serve(gate.clone()).await;
    let second = serve(gate.clone()).await;
    let harness = start_harness("hls-parallel-ready", DeliveryOptions::default());
    let mut items = vec![
        sized_item("first", &first, 32, 4_000),
        sized_item("second", &second, 32, 4_000),
    ];
    for item in &mut items {
        item.meta.delivery = DeliveryKind::Hls;
    }
    harness.handle.update_focus(focus_now(items, 0, 0));

    wait_for_starts(
        &harness,
        &gate,
        &["first", "second"],
        "both HLS root requests start",
    )
    .await;
    assert_eq!(
        harness.segmented.snapshot("first").phase,
        SegmentedPhase::Preparing
    );
    assert_eq!(
        harness.segmented.snapshot("second").phase,
        SegmentedPhase::Preparing
    );
    gate.release.add_permits(2);
    wait_ready(&harness, "first").await;
    wait_ready(&harness, "second").await;
    assert!(harness.segmented.object(&first).is_some());
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_ready(harness: &delivery_fixture::DeliveryHarness, post: &str) {
    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, post).await;
    assert_eq!(
        terminal.phase,
        SegmentedPhase::Ready,
        "all selected HLS dependencies become ready"
    );
}
