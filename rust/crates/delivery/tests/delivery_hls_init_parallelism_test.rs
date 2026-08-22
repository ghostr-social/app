mod delivery_fixture;
mod hls_terminal_wait;

use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use std::time::Duration;

#[tokio::test]
async fn authorized_hls_initialization_stages_prepare_in_parallel() {
    let first_gate = HlsGate::blocking("init");
    let second_gate = HlsGate::blocking("init");
    let first = serve(first_gate.clone()).await;
    let second = serve(second_gate.clone()).await;
    let harness = start_harness("hls-parallel-init", DeliveryOptions::default());
    let mut items = vec![item("first", &first), item("second", &second)];
    for item in &mut items {
        item.meta.delivery = DeliveryKind::Hls;
    }
    harness.handle.update_focus(focus_now(items, 0, 0));

    let both_started = async {
        first_gate.started.acquire().await.unwrap().forget();
        second_gate.started.acquire().await.unwrap().forget();
    };
    tokio::time::timeout(Duration::from_secs(2), both_started)
        .await
        .expect("both authorized HLS init stages start before either completes");
    first_gate.release.add_permits(1);
    second_gate.release.add_permits(1);
    assert_eq!(ready(&harness, "first").await, SegmentedPhase::Ready);
    assert_eq!(ready(&harness, "second").await, SegmentedPhase::Ready);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn item(id: &'static str, source: &str) -> ghostr_delivery::delivery_events::FocusItem {
    sized_item(id, source, 32, 4_000)
}

async fn ready(harness: &delivery_fixture::DeliveryHarness, post: &str) -> SegmentedPhase {
    hls_terminal_wait::wait_terminal(&harness.segmented, post)
        .await
        .phase
}
