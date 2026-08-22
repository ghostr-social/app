mod delivery_fixture;

use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::DeliveryKind;
use std::num::NonZeroUsize;
use std::time::Duration;

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

    started(&gate).await;
    let second = tokio::time::timeout(Duration::from_millis(100), gate.started.acquire()).await;
    assert!(
        second.is_err(),
        "one slow origin consumed another origin slot"
    );
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn item(id: &'static str, source: &str) -> ghostr_delivery::delivery_events::FocusItem {
    sized_item(id, source, 32, 4_000)
}

async fn started(gate: &HlsGate) {
    tokio::time::timeout(Duration::from_secs(2), gate.started.acquire())
        .await
        .expect("first HLS request starts")
        .unwrap()
        .forget();
}
