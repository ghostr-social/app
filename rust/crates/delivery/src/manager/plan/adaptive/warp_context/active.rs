use super::{hedge, PlanInputs};
use crate::manager::hedge_tail::HedgeTailWake;
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    ActivePlannerContext, AllocationPlan, FeedOffset, PlannerContext, SoftRequestCommitment,
};

#[cfg(test)]
#[path = "active/commitment_test.rs"]
mod commitment_test;

#[derive(Clone, Copy)]
pub(super) struct BuildInput<'a> {
    pub(super) state: &'a DeliveryState,
    pub(super) snapshot: &'a ghostr_engine::adaptive::PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) inputs: &'a PlanInputs<'a>,
}

#[derive(Clone, Copy)]
pub(super) struct ActiveContextInput<'a> {
    pub(super) state: &'a DeliveryState,
    pub(super) snapshot: &'a ghostr_engine::adaptive::PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) inputs: &'a PlanInputs<'a>,
    pub(super) active: &'a ActiveAction,
}

pub(super) fn apply(
    mut context: PlannerContext,
    input: BuildInput<'_>,
) -> (
    PlannerContext,
    Vec<HedgeTailWake>,
    Vec<SoftRequestCommitment>,
) {
    let mut tails = Vec::new();
    let mut soft = Vec::new();
    for active in input.inputs.in_flight {
        let evidence = ActiveContextInput {
            state: input.state,
            snapshot: input.snapshot,
            base: input.base,
            inputs: input.inputs,
            active,
        };
        let (active, tail, commitment) = context_for(evidence);
        context = context.with_active(active);
        tails.extend(tail);
        soft.extend(commitment);
    }
    (context, tails, soft)
}

fn context_for(
    input: ActiveContextInput<'_>,
) -> (
    ActivePlannerContext,
    Option<HedgeTailWake>,
    Option<SoftRequestCommitment>,
) {
    let active = input.active;
    let advantage = continuation_advantage(input, active.action_id());
    let context = ActivePlannerContext::new(active.action_id(), active.post().clone())
        .with_continuation_advantage(advantage);
    let hedge = hedge::apply(context, &input);
    let context = if active.cancelling() {
        hedge.context.mark_cancelling()
    } else {
        hedge.context
    };
    (context, hedge.wake, hedge.soft)
}

fn continuation_advantage(input: ActiveContextInput<'_>, action: ghostr_engine::ActionId) -> i64 {
    let retained = input
        .base
        .retained
        .iter()
        .any(|item| item.action_id == action);
    let protected_commitment = input
        .snapshot
        .candidates
        .iter()
        .find(|candidate| candidate.post == *input.active.post())
        .is_some_and(|candidate| {
            unexpired_nonhistorical_commitment(
                candidate.feed_offset,
                input.active.committed_until_ms(),
                input.snapshot.observed_at_ms,
            )
        });
    if retained || protected_commitment {
        100_000
    } else {
        -100_000
    }
}

fn unexpired_nonhistorical_commitment(
    offset: FeedOffset,
    committed_until_ms: u64,
    observed_at_ms: u64,
) -> bool {
    offset.value() >= 0 && committed_until_ms > observed_at_ms
}
