mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use ghostr_engine::adaptive::DecisionOutcome;

const UNREACHABLE: &str = "http://127.0.0.1:9/video.mp4";

#[tokio::test]
async fn terminal_noop_plan_links_its_exact_decision_sequence() {
    let harness = start_harness("planning-cycle-link", DeliveryOptions::default());
    let item = sized_item("ready", UNREACHABLE, 16, 1_000);
    seed_range(&harness.store, &item, 0, &[1; 16]).await;

    harness
        .handle
        .update_focus(focus_now(vec![item], 0, 1_000));

    let plan = wait_for_linked_plan(&harness.handle).await;
    let decision = harness
        .handle
        .decision_history()
        .records
        .into_iter()
        .find(|record| Some(record.sequence) == plan.decision_sequence)
        .expect("linked decision must remain observable");
    assert_eq!(decision.eventual_outcome, DecisionOutcome::Succeeded {
        bytes: 0,
        elapsed_ms: 0,
    });
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_linked_plan(handle: &DeliveryHandle) -> PlanEvidence {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(plan) = handle
                .plan_history()
                .into_iter()
                .find(|plan| plan.decision_sequence.is_some())
            {
                return plan;
            }
            handle.plan_notifier().notified().await;
        }
    })
    .await
    .expect("linked planning cycle")
}
