use super::{hedge, PlanInputs};
use crate::manager::hedge_tail::HedgeTailWake;
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    ActivePlannerContext, AllocationPlan, FeedOffset, PlannerContext, SoftRequestCommitment,
};

const CONTINUE_ADVANTAGE: i64 = 100_000;
const ABORT_ADVANTAGE: i64 = -100_000;
const FINISH_BLOCK_ADVANTAGE: i64 = 0;

#[cfg(test)]
#[path = "active/commitment_test.rs"]
mod commitment_test;
mod handoff;

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
    let handoff = handoff::Admission::build(handoff::Input {
        state: input.state,
        snapshot: input.snapshot,
        base: input.base,
        inputs: input.inputs,
    });
    for active in input.inputs.in_flight {
        let evidence = ActiveContextInput {
            state: input.state,
            snapshot: input.snapshot,
            base: input.base,
            inputs: input.inputs,
            active,
        };
        let (active, tail, commitment) = context_for(evidence, &handoff);
        context = context.with_active(active);
        tails.extend(tail);
        soft.extend(commitment);
    }
    (context, tails, soft)
}

fn context_for(
    input: ActiveContextInput<'_>,
    handoff: &handoff::Admission,
) -> (
    ActivePlannerContext,
    Option<HedgeTailWake>,
    Option<SoftRequestCommitment>,
) {
    let active = input.active;
    let advantage = continuation_advantage(input, handoff, active.action_id());
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

fn continuation_advantage(
    input: ActiveContextInput<'_>,
    handoff: &handoff::Admission,
    action: ghostr_engine::ActionId,
) -> i64 {
    if input.active.io_finished() {
        return CONTINUE_ADVANTAGE;
    }
    let retained = input
        .base
        .retained
        .iter()
        .any(|item| item.action_id == action);
    if retained {
        return CONTINUE_ADVANTAGE;
    }
    let protected = input.snapshot.candidates.iter().find_map(|candidate| {
        candidate
            .in_flight
            .iter()
            .find(|active| active.action_id == action)
            .map(|active| (candidate.feed_offset, active))
    });
    if protected.is_some_and(|(offset, active)| {
        unexpired_current_commitment(
            offset,
            active.identity_current,
            active.committed_until_ms,
            input.snapshot.observed_at_ms,
        )
    }) {
        return CONTINUE_ADVANTAGE;
    }
    if handoff.permits(action) {
        FINISH_BLOCK_ADVANTAGE
    } else {
        ABORT_ADVANTAGE
    }
}

fn unexpired_current_commitment(
    offset: FeedOffset,
    identity_current: bool,
    committed_until_ms: u64,
    observed_at_ms: u64,
) -> bool {
    identity_current && offset.value() >= 0 && committed_until_ms > observed_at_ms
}
