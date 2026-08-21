use ghostr_delivery::delivery_events::{DecisionHistorySnapshot, DeliveryHandle};
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord};
use std::time::Duration;

pub async fn wait_for_completed_bytes(handle: &DeliveryHandle, expected: u64) -> DecisionRecord {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(record) = handle.decision_history().records.iter().find(|record| {
                record.chosen_action_id.is_some()
                    && matches!(
                        record.eventual_outcome,
                        DecisionOutcome::Succeeded { bytes, .. } if bytes == expected
                    )
            }) {
                return record.clone();
            }
            changed.await;
        }
    })
    .await
    .expect("completed bytes were not bound to the selected decision")
}

pub async fn wait_for_history(
    handle: &DeliveryHandle,
    ready: impl Fn(&DecisionHistorySnapshot) -> bool,
) {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if ready(&handle.decision_history()) {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("decision history transition");
}
