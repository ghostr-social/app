mod delivery_fixture;

use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::DeliveryKind;
use std::time::Duration;

#[tokio::test]
async fn severe_packet_loss_keeps_hls_bootstrap_at_one_request() {
    let gate = HlsGate::new();
    let first = serve(gate.clone()).await;
    let second = serve(gate.clone()).await;
    let harness = start_harness("hls-severe-loss", DeliveryOptions::default());
    harness.network.update(NetworkProfile {
        packet_loss_bps: 6_000,
        ..NetworkProfile::default()
    });
    harness.handle.network_changed();
    let mut items = vec![item("first", &first), item("second", &second)];
    for item in &mut items {
        item.meta.delivery = DeliveryKind::Hls;
    }
    harness.handle.update_focus(focus_now(items, 0, 0));

    started(&gate).await;
    let second = tokio::time::timeout(Duration::from_millis(100), gate.started.acquire()).await;
    assert!(second.is_err(), "severe loss admitted parallel HLS work");
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
