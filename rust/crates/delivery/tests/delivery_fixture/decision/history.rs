use super::super::evidence::DeliveryEvidence as _;
use core::time::Duration;
use ghostr_delivery::delivery_events::{DecisionHistorySnapshot, DeliveryHandle};

const WAIT_LIMIT: Duration = Duration::from_secs(10);

pub async fn wait_for_history(
    handle: &DeliveryHandle,
    ready: impl Fn(&DecisionHistorySnapshot) -> bool,
) {
    wait_for_history_with_limit(handle, WAIT_LIMIT, ready).await;
}

pub async fn wait_for_history_with_limit(
    handle: &DeliveryHandle,
    limit: Duration,
    ready: impl Fn(&DecisionHistorySnapshot) -> bool,
) {
    let notifier = handle.plan_notifier();
    let transition = tokio::time::timeout(limit, async {
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
    .await;
    transition.unwrap_or_else(|error| panic_timeout(error, handle));
}

fn panic_timeout(error: tokio::time::error::Elapsed, handle: &DeliveryHandle) -> ! {
    let history = handle.decision_history();
    let last = history
        .records
        .last()
        .map(|record| (record.sequence, &record.eventual_outcome));
    panic!(
        "decision history transition: {error:?}; records={}; last={last:?}",
        history.records.len()
    );
}
