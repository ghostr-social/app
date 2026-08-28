use crate::adaptive::{
    ActionKind, ActivePlannerContext, HedgeInput, IdentityProof, PlannerCapability, PlannerContext,
    TransformCapability, TransformKind,
};
use crate::{ActionId, ByteRange, PostId};

pub(super) fn context(
    input: &crate::adaptive::PlayabilitySnapshot,
    action: ActionId,
    post: &PostId,
    mirror: &str,
) -> PlannerContext {
    let hedge = HedgeInput::new(action, ActionKind::FetchRange(ByteRange::new(0, 64_000)))
        .with_timing(1_000, 900)
        .with_value(5_000, 1_000)
        .with_network_envelope(800_000);
    let active = ActivePlannerContext::new(action, post.clone())
        .with_continuation_advantage(-100_000)
        .with_hedge(hedge, IdentityProof::VerifiedHash([3; 32]), mirror);
    PlannerContext::explicitly_unavailable(input)
        .with_capability(
            post,
            PlannerCapability::reported(
                false,
                Some(TransformCapability::new(TransformKind::Remux, 300, 900_000)),
                4,
            ),
        )
        .with_active(active)
}
