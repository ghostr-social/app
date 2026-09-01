use ghostr_delivery::delivery_events::{DecisionHistorySnapshot, DeliveryHandle};
use ghostr_engine::adaptive::DecisionOutcome;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct DecisionEnvelope {
    decisions: DecisionHistorySnapshot,
}

pub fn pending_transfer_sequence(handle: &DeliveryHandle) -> u64 {
    history(handle)
        .records
        .into_iter()
        .filter(|record| {
            record.chosen_action_id.is_some() && record.eventual_outcome == DecisionOutcome::Pending
        })
        .max_by_key(|record| record.sequence)
        .expect("bound speculative prefix transfer")
        .sequence
}

pub async fn wait_for_zero_byte_cancellation(handle: &DeliveryHandle, sequence: u64) {
    let notifier = handle.plan_notifier();
    let observed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let notified = notifier.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if cancelled(handle, sequence) {
                return;
            }
            notified.await;
        }
    })
    .await;
    assert!(
        observed.is_ok(),
        "decision {sequence} never cancelled at zero bytes; history={:#?}",
        history(handle)
    );
}

fn cancelled(handle: &DeliveryHandle, sequence: u64) -> bool {
    history(handle).records.iter().any(|record| {
        record.sequence == sequence
            && matches!(
                record.eventual_outcome,
                DecisionOutcome::Cancelled { bytes: 0, .. }
            )
    })
}

fn history(handle: &DeliveryHandle) -> DecisionHistorySnapshot {
    let json = handle
        .decision_history_json()
        .expect("decision evidence JSON");
    serde_json::from_str::<DecisionEnvelope>(&json)
        .expect("decision evidence schema")
        .decisions
}
