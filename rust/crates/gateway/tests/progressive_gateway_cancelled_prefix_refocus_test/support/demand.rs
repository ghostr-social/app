use crate::gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use ghostr_delivery::playback_demand::{ConsumerId, DemandState};
use std::time::Duration;

pub async fn wait_for_blocked(
    harness: &ProgressiveDeliveryHarness,
    excluding: Option<ConsumerId>,
    label: &'static str,
) -> ConsumerId {
    wait(harness, label, |state| match state {
        DemandState::Blocked(lease)
            if lease.post().as_str() == "p6" && Some(lease.consumer()) != excluding =>
        {
            Some(lease.consumer())
        }
        _ => None,
    })
    .await
}

async fn wait<T: Copy>(
    harness: &ProgressiveDeliveryHarness,
    label: &'static str,
    match_state: impl Fn(&DemandState) -> Option<T>,
) -> T {
    let observed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(value) = harness.delivery.demands().iter().find_map(&match_state) {
                return value;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    observed.unwrap_or_else(|_| demand_timeout(harness, label))
}

fn demand_timeout<T>(harness: &ProgressiveDeliveryHarness, label: &str) -> T {
    panic!(
        "absent demand transition: {label}; demands={:#?}; latest_plan={:#?}",
        harness.delivery.demands(),
        harness.delivery.handle.latest_plan(),
    )
}
