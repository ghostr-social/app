use super::*;
use crate::adaptive::{
    HlsGenerationPolicy, OriginAdmissionGenerationPolicy, PromotionGenerationPolicy,
    RangeAliasGenerationPolicy,
};

impl WarpPlanner {
    pub(crate) fn plan_legacy_hls_for_test(
        &mut self,
        input: WarpPlannerInput<'_>,
    ) -> WarpPlanningDecision {
        self.plan_with_generation_policies(
            input,
            WarpGenerationPolicies {
                hls: HlsGenerationPolicy::LegacyWholeStage,
                ..WarpGenerationPolicies::current()
            },
        )
    }

    pub(crate) fn plan_legacy_promotions_for_test(
        &mut self,
        input: WarpPlannerInput<'_>,
    ) -> WarpPlanningDecision {
        self.plan_with_generation_policies(
            input,
            WarpGenerationPolicies {
                promotion: PromotionGenerationPolicy::LegacyLatentGrant,
                ..WarpGenerationPolicies::current()
            },
        )
    }

    pub(crate) fn plan_legacy_origin_admission_for_test(
        &mut self,
        input: WarpPlannerInput<'_>,
    ) -> WarpPlanningDecision {
        self.plan_with_generation_policies(
            input,
            WarpGenerationPolicies {
                origin_admission: OriginAdmissionGenerationPolicy::LegacyUnclassified,
                ..WarpGenerationPolicies::current()
            },
        )
    }

    pub(crate) fn plan_legacy_range_aliases_for_test(
        &mut self,
        input: WarpPlannerInput<'_>,
    ) -> WarpPlanningDecision {
        self.plan_with_generation_policies(
            input,
            WarpGenerationPolicies {
                range_alias: RangeAliasGenerationPolicy::LegacyIndependentActions,
                ..WarpGenerationPolicies::current()
            },
        )
    }
}

impl PlannerReplayCapsule {
    pub(crate) fn mark_incomplete(&mut self) {
        self.complete = false;
    }
    pub(crate) fn base_mut(&mut self) -> &mut AllocationPlan {
        &mut self.base
    }
    pub(crate) fn origins_mut(&mut self) -> &mut OriginModel {
        &mut self.origins
    }
    pub(crate) fn context_mut(&mut self) -> &mut PlannerContext {
        &mut self.context
    }
    pub(crate) fn config_mut(&mut self) -> &mut WarpPlannerConfig {
        &mut self.config
    }
    pub(crate) fn controller_prices_mut(&mut self) -> &mut ResourcePrices {
        &mut self.controller_prices
    }
    pub(crate) fn network_mut(&mut self) -> Option<&mut NetworkTokenBucket> {
        self.network.as_mut()
    }
    pub(crate) fn price_epoch_mut(&mut self) -> &mut u64 {
        &mut self.price_epoch
    }
}
