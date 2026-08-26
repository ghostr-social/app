use super::{GeneratedActions, HlsGenerationPolicy, WarpActionGenerator, WarpGenerationInput};
use crate::adaptive::{AllocationPlan, PlannerContext, PlayabilitySnapshot};
use crate::origin_model::OriginModel;

impl WarpActionGenerator {
    pub(crate) fn generate(
        snapshot: &PlayabilitySnapshot,
        base: &AllocationPlan,
        origins: &OriginModel,
        context: &PlannerContext,
    ) -> GeneratedActions {
        Self::generate_with_policy(
            WarpGenerationInput::new(snapshot, base, origins, context),
            HlsGenerationPolicy::BoundedObjectCursor,
        )
    }
}
