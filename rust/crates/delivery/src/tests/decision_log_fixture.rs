use crate::delivery_events::{
    CommandReceiver, DecisionHistorySnapshot, DecisionToken, DeliveryHandle,
    LegacyDecisionPublication, WarpDecisionPublication,
};
use crate::manager::plan::PlannedWork;
use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::{AllocationPlan, DecisionOutcome, StorageSnapshot};

#[cfg(test)]
#[path = "decision_history_warp_noop_retention_test.rs"]
mod warp_noop_retention_test;

#[cfg(test)]
#[path = "decision_claim_lifecycle_test.rs"]
mod claim_lifecycle_test;

#[cfg(test)]
#[path = "decision_probe_claim_eligibility_test.rs"]
mod claim_eligibility_test;

mod head;

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

pub(crate) fn selected_head(
    _handle: &DeliveryHandle,
    commands: &CommandReceiver,
) -> (u64, DecisionToken) {
    let work = head::work();
    let published = commands.publish_warp_decision(WarpDecisionPublication {
        snapshot: work.snapshot.as_ref().expect("planning snapshot"),
        decision: work.warp.as_ref().expect("WARP decision"),
        legacy_prices: work.shadow_prices,
        models: &work.decision_models,
    });
    let sequence = published.sequence();
    let token = published
        .into_token()
        .expect("selected HEAD decision token");
    (sequence, token)
}

pub(crate) use head::{
    identity as head_identity, work as head_work, wrong_post_identity, wrong_source_identity,
};

pub(crate) fn selected_warp(
    _handle: &DeliveryHandle,
    commands: &CommandReceiver,
    work: &PlannedWork,
) -> (u64, DecisionToken) {
    let published = commands.publish_warp_decision(WarpDecisionPublication {
        snapshot: work.snapshot.as_ref().expect("planning snapshot"),
        decision: work.warp.as_ref().expect("WARP decision"),
        legacy_prices: work.shadow_prices,
        models: &work.decision_models,
    });
    let sequence = published.sequence();
    let token = published
        .into_token()
        .expect("selected WARP decision token");
    (sequence, token)
}

pub(crate) fn publish(
    _handle: &DeliveryHandle,
    commands: &CommandReceiver,
    work: &PlannedWork,
    allocation: &AllocationPlan,
) -> (u64, Option<DecisionToken>) {
    let published = commands.publish_decision(LegacyDecisionPublication {
        snapshot: work.snapshot.as_ref().expect("planning snapshot"),
        plan: allocation,
        prices: work.shadow_prices,
        models: &work.decision_models,
    });
    let sequence = published.sequence();
    (sequence, published.into_token())
}

pub(crate) fn outcome(history: &DecisionHistorySnapshot, sequence: u64) -> &DecisionOutcome {
    &history
        .records
        .iter()
        .find(|record| record.sequence == sequence)
        .expect("decision record")
        .eventual_outcome
}
