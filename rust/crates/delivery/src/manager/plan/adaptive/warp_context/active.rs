use super::{hedge, PlanInputs};
use crate::manager::hedge_tail::HedgeTailWake;
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    ActivePlannerContext, AllocationPlan, PlannerContext, SoftRequestCommitment,
};

pub(super) struct BuildInput<'a> {
    pub(super) state: &'a DeliveryState,
    pub(super) snapshot: &'a ghostr_engine::adaptive::PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) inputs: &'a PlanInputs<'a>,
}

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
    let advantage = continuation_advantage(input.base, active.action_id());
    let context = ActivePlannerContext::new(active.action_id(), active.post().clone())
        .with_continuation_advantage(advantage);
    let hedge = hedge::apply(context, &input);
    let context = match active.cancelling() {
        true => hedge.context.mark_cancelling(),
        false => hedge.context,
    };
    (context, hedge.wake, hedge.soft)
}

fn continuation_advantage(base: &AllocationPlan, action: ghostr_engine::ActionId) -> i64 {
    match base.retained.iter().any(|item| item.action_id == action) {
        true => 100_000,
        false => -100_000,
    }
}
