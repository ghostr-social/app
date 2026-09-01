use crate::delivery_fixture::evidence::DeliveryEvidence as _;
use crate::delivery_fixture::transient_origin::{body_count, Attempts};
use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use ghostr_engine::adaptive::DecisionOutcome;

const WAIT_LIMIT: Duration = Duration::from_secs(30);

pub async fn wait_for_attempts(attempts: &Attempts, expected: usize) {
    tokio::time::timeout(WAIT_LIMIT, async {
        while body_count(attempts) < expected {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for attempt {expected}"));
}

pub async fn wait_for_failures(handle: &DeliveryHandle, expected: usize) -> u64 {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(WAIT_LIMIT, async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let failure = handle
                .decision_history()
                .records
                .into_iter()
                .filter(failed_transfer)
                .nth(expected - 1);
            if let Some(record) = failure {
                return record.sequence;
            }
            changed.await;
        }
    })
    .await
    .unwrap_or_else(|_| failure_timeout(handle, expected))
}

pub async fn wait_for_decision_successor(handle: &DeliveryHandle, sequence: u64) -> PlanEvidence {
    wait_for_plan(handle, |plan| {
        plan.decision_sequence
            .is_some_and(|decision| decision > sequence)
    })
    .await
}

pub async fn wait_for_focus(handle: &DeliveryHandle, generation: u64) -> PlanEvidence {
    wait_for_plan(handle, |plan| plan.focus_generation == Some(generation)).await
}

fn failure_timeout(handle: &DeliveryHandle, expected: usize) -> ! {
    let history = handle.decision_history();
    let observed = history
        .records
        .iter()
        .filter(|record| failed_transfer(record))
        .count();
    panic!(
        "expected {expected} transfer failures; observed {observed} across {} decisions",
        history.records.len()
    );
}

fn failed_transfer(record: &ghostr_engine::adaptive::DecisionRecord) -> bool {
    record.executed_request.is_some()
        && matches!(record.eventual_outcome, DecisionOutcome::Failed { .. })
}

async fn wait_for_plan(
    handle: &DeliveryHandle,
    ready: impl Fn(&PlanEvidence) -> bool,
) -> PlanEvidence {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(WAIT_LIMIT, async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(plan) = handle.latest_plan().filter(&ready) {
                return plan;
            }
            changed.await;
        }
    })
    .await
    .expect("causally subsequent plan")
}
