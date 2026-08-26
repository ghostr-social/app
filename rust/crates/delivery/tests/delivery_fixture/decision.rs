use super::evidence::DeliveryEvidence as _;
use core::time::Duration;
use ghostr_delivery::delivery_events::{DecisionHistorySnapshot, DeliveryHandle};
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord};

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

pub async fn wait_for_terminal_transfer(handle: &DeliveryHandle) -> DecisionRecord {
    wait_for_history(handle, |history| {
        history.records.iter().any(terminal_transfer)
    })
    .await;
    handle
        .decision_history()
        .records
        .into_iter()
        .rev()
        .find(terminal_transfer)
        .expect("terminal transfer decision")
}

fn terminal_transfer(record: &DecisionRecord) -> bool {
    record.eventual_outcome != DecisionOutcome::Pending
        && matches!(
            record
                .warp_decision
                .as_ref()
                .and_then(|decision| decision.selected.as_ref())
                .map(|selected| &selected.command),
            Some(ghostr_engine::adaptive::RecordedWarpCommand::Transfer { .. })
        )
}

pub async fn wait_for_promotion(handle: &DeliveryHandle) {
    wait_for_history(handle, |history| {
        history.records.iter().any(|record| {
            matches!(
                record.eventual_outcome,
                DecisionOutcome::Succeeded { bytes: 0, .. }
            ) && matches!(
                record
                    .warp_decision
                    .as_ref()
                    .and_then(|decision| decision.selected.as_ref())
                    .map(|selected| &selected.command),
                Some(ghostr_engine::adaptive::RecordedWarpCommand::Promote { .. })
            )
        })
    })
    .await;
}
