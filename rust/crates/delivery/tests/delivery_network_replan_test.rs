mod delivery_fixture;

use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use std::time::Duration;

#[tokio::test]
async fn network_change_wakes_an_idle_manager_for_replanning() {
    let harness = start_harness("ghostr-network-replan", DeliveryOptions::default());
    assert!(harness.handle.plan_history().is_empty());
    harness.network.update(lossy_profile());

    harness.handle.network_changed();

    let plan = wait_for_plan(&harness.handle).await;
    assert_eq!(plan.revision, 1);
}

fn lossy_profile() -> NetworkProfile {
    NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        packet_loss_bps: 6_000,
        max_connections_per_host: 0,
    }
}

async fn wait_for_plan(handle: &DeliveryHandle) -> PlanEvidence {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if let Some(plan) = handle.plan_history().into_iter().next() {
                return plan;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("network change did not produce a plan")
}
