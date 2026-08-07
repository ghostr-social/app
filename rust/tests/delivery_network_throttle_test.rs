mod support;

use rust_lib_ghostr::video::debug_network::NetworkProfile;
use std::time::Duration;
use support::delivery::start_harness;
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_options::DeliveryOptions;
use support::delivery_wait::wait_for_ranges;

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
