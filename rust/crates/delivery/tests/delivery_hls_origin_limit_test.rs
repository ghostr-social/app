mod delivery_fixture;

use core::num::NonZeroUsize;
use core::time::Duration;
use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::hls_start::wait_for_start;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn hls_bootstraps_respect_the_configured_per_origin_limit() {
    let gate = HlsGate::new();
    let source = serve(gate.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.max_requests_per_authority = NonZeroUsize::new(1);
    let harness = start_harness("hls-origin-limit", options);
    let mut items = vec![item("first", &source), item("second", &source)];
    for item in &mut items {
        item.meta.delivery = DeliveryKind::Hls;
    }
    harness.handle.update_focus(focus_now(items, 0, 0));

    wait_for_start(&harness, &gate, "first", "first HLS request starts").await;
    let second = tokio::time::timeout(Duration::from_millis(100), gate.started.acquire()).await;
    assert!(
        second.is_err(),
        "one slow origin consumed another origin slot"
    );
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn item(id: &'static str, source: &str) -> ghostr_delivery::delivery_events::FocusItem {
    sized_item(id, source, 32, 4_000)
}
