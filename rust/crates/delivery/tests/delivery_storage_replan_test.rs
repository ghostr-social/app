mod delivery_fixture;

use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use std::time::Duration;

#[tokio::test]
async fn storage_change_wakes_an_idle_manager_for_replanning() {
    let harness = start_harness("ghostr-storage-replan", DeliveryOptions::default());
    assert!(harness.handle.plan_history().is_empty());

    harness.handle.storage_changed();

    let plan = wait_for_plan(&harness.handle).await;
    assert_eq!(plan.revision, 1);
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
    .expect("storage change did not produce a plan")
}
