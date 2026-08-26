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
        .map(|candidate| &candidate.post)
        .chain(
            snapshot
                .hls_candidates
                .iter()
                .map(|candidate| &candidate.post),
        )
        .fold(context, |context, post| {
            let Some(eligible_at_ms) = inputs.retry.cooling_until(post) else {
                return context;
            };
            context
                .with_retry_availability(post, PlannerRetryAvailability::Cooling { eligible_at_ms })
        })
}
