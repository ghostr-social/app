use super::super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{CandidateSnapshot, PlannerContext};

pub(super) fn apply(
    context: PlannerContext,
    state: &DeliveryState,
    candidate: &CandidateSnapshot,
    inputs: &PlanInputs<'_>,
) -> PlannerContext {
    let Some(source) = candidate.preferred_source.as_deref() else {
        return context;
    };
    let Some(identity) = state.catalog().transfer_identity(&candidate.post, source) else {
        return context;
    };
    let Some(exhaustion) = inputs.whole_body_exhaustions.get(&identity) else {
        return context;
    };
    context.with_whole_body_exhaustion(&candidate.post, *exhaustion)
}
