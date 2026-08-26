use super::*;

impl WarpPlanner {
    pub(crate) fn plan_legacy_hls_for_test(
        &mut self,
        input: WarpPlannerInput<'_>,
    ) -> WarpPlanningDecision {
        self.plan_with_hls_policy(input, HlsGenerationPolicy::LegacyWholeStage)
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
