//! Runtime policy changes update the exact executor shared by media workers.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::DataUsageLevel;
use ghostr_net::media_request_executor::MediaRequestLimits;

#[tokio::test]
async fn data_usage_and_network_changes_synchronize_request_gate_limits() {
    let harness = start_harness("request-gate-limits", DeliveryOptions::default());
    wait_limits(&harness.requests, limits(3, 3)).await;

    harness.handle.set_data_usage(DataUsageLevel::Aggressive);
    wait_limits(&harness.requests, limits(4, 4)).await;

    let _ = harness.handle.update_network_profile(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        packet_loss_bps: 0,
        max_connections_per_host: 1,
    });
    wait_limits(&harness.requests, limits(4, 1)).await;
}

async fn wait_limits(
    requests: &ghostr_net::media_request_executor::MediaRequestExecutor,
    expected: MediaRequestLimits,
) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while requests.limits() != expected {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("manager did not initialize request limits");
}

fn limits(global: usize, per_authority: usize) -> MediaRequestLimits {
    MediaRequestLimits::try_new(global, per_authority).expect("valid test fixture")
}
