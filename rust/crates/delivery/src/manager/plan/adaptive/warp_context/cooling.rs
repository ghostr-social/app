use super::super::super::PlanInputs;
use ghostr_engine::adaptive::{PlannerContext, PlannerRetryAvailability, PlayabilitySnapshot};

pub(super) fn apply(
    context: PlannerContext,
    snapshot: &PlayabilitySnapshot,
    inputs: &PlanInputs<'_>,
) -> PlannerContext {
    snapshot
        .candidates
        .iter()
        .fold(context, |context, candidate| {
            let Some(eligible_at_ms) = inputs.retry.cooling_until(&candidate.post) else {
                return context;
            };
            context.with_retry_availability(
                candidate.post.clone(),
                PlannerRetryAvailability::Cooling { eligible_at_ms },
            )
        })
}
