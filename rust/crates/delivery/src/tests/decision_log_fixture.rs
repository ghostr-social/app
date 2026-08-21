use crate::delivery_events::{
    CommandReceiver, DecisionHistorySnapshot, DecisionToken, DeliveryHandle,
    LegacyDecisionPublication,
};
use crate::manager::plan::PlannedWork;
use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::{AllocationPlan, DecisionOutcome, StorageSnapshot};

#[cfg(test)]
#[path = "decision_history_warp_noop_retention_test.rs"]
mod warp_noop_retention_test;

pub(crate) fn work() -> PlannedWork {
    plan(20_000, 4_000_000, StorageSnapshot::new(2_000_000_000, 0))
}

pub(crate) fn selected(
    handle: &DeliveryHandle,
    commands: &CommandReceiver,
    work: &PlannedWork,
) -> (u64, DecisionToken) {
    let (sequence, token) = publish(handle, commands, work, &work.plan);
    (sequence, token.expect("selected decision token"))
}

pub(crate) fn publish(
    handle: &DeliveryHandle,
    commands: &CommandReceiver,
    work: &PlannedWork,
    allocation: &AllocationPlan,
) -> (u64, Option<DecisionToken>) {
    let token = commands.publish_decision(LegacyDecisionPublication {
        snapshot: work.snapshot.as_ref().expect("planning snapshot"),
        plan: allocation,
        prices: work.shadow_prices,
        models: &work.decision_models,
    });
    let sequence = handle.decision_history().records.last().unwrap().sequence;
    (sequence, token)
}

pub(crate) fn outcome(history: &DecisionHistorySnapshot, sequence: u64) -> &DecisionOutcome {
    &history
        .records
        .iter()
        .find(|record| record.sequence == sequence)
        .expect("decision record")
        .eventual_outcome
}
