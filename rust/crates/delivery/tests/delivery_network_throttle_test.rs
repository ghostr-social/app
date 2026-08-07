mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_delivery::debug::network::NetworkProfile;
use std::time::Duration;

#[tokio::test]
async fn live_network_profile_delays_real_delivery_manager_transfers() {
    let origin = serve_recording("throttled", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-network-throttle", DeliveryOptions::default());
    harness.network.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 250,
        max_connections_per_host: 0,
    });

    harness.handle.update_focus(focus_now(
        vec![sized_item("clip", &origin, 16, 1_000)],
        0,
        0,
    ));
    tokio::time::sleep(Duration::from_millis(50)).await;

    let early = harness.store.present_ranges("clip").await.expect("ranges");
    assert!(
        early.is_empty(),
        "configured latency must precede the request"
    );
    wait_for_ranges(&harness.store, "clip", &[(0, 16)]).await;
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
