use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{PlannerContext, PlannerQuality};
use ghostr_engine::PostId;

pub(super) fn apply(
    context: PlannerContext,
    state: &DeliveryState,
    post: &PostId,
) -> PlannerContext {
    let Some(evidence) = state.catalog().rendition_quality(post) else {
        return context;
    };
    context.with_quality(post.clone(), PlannerQuality::from_rendition(evidence))
}
