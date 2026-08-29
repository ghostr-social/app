mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::hls_start::wait_for_start;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn severe_packet_loss_keeps_hls_bootstrap_at_one_request() {
    let gate = HlsGate::new();
    let first = serve(gate.clone()).await;
    let second = serve(gate.clone()).await;
    let harness = start_harness("hls-severe-loss", DeliveryOptions::default());
    let _ = harness.handle.update_network_profile(NetworkProfile {
        packet_loss_bps: 6_000,
        ..NetworkProfile::default()
    });
    let mut items = vec![item("first", &first), item("second", &second)];
    for item in &mut items {
        item.meta.delivery = DeliveryKind::Hls;
    }
    harness.handle.update_focus(focus_now(items, 0, 0));

    wait_for_start(&harness, &gate, "first", "first HLS request starts").await;
    let second = tokio::time::timeout(Duration::from_millis(100), gate.started.acquire()).await;
    assert!(second.is_err(), "severe loss admitted parallel HLS work");
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn item(id: &'static str, source: &str) -> ghostr_delivery::delivery_events::FocusItem {
    sized_item(id, source, 32, 4_000)
}
