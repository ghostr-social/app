mod delivery_fixture;

use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use std::time::Duration;

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

    tokio::time::timeout(Duration::from_secs(2), gate.started.acquire_many(2))
        .await
        .unwrap()
        .unwrap()
        .forget();
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
    let changed = harness.segmented.notifier();
    let result = tokio::time::timeout(Duration::from_secs(2), async {
        while harness.segmented.snapshot(post).phase != SegmentedPhase::Ready {
            changed.notified().await;
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "{post} did not become ready: {:?}",
        harness.segmented.snapshot(post)
    );
}
