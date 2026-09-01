use super::{WarpPlanner, WarpPlannerConfig, WarpPlannerInput};
use crate::adaptive::{
    AllocationPlan, NetworkTokenBucket, PlannerContext, ResourcePrices, WarpGenerationPolicies,
};
use crate::origin_model::OriginModel;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlannerReplayCapsule {
    complete: bool,
    base: AllocationPlan,
    origins: OriginModel,
    context: PlannerContext,
    config: WarpPlannerConfig,
    controller_prices: ResourcePrices,
    network: Option<NetworkTokenBucket>,
    price_epoch: u64,
    last_feedback: Option<crate::adaptive::ResourceFeedback>,
    generation_policies: WarpGenerationPolicies,
    sources: Vec<String>,
}

impl PlannerReplayCapsule {
    pub(super) fn capture(
        input: &WarpPlannerInput<'_>,
        planner: &WarpPlanner,
        generation_policies: WarpGenerationPolicies,
    ) -> Self {
        Self {
            complete: true,
            base: input.base.clone(),
            origins: input.origins.clone(),
            context: input.context.clone(),
            config: planner.config,
            controller_prices: planner.prices.prices(),
            network: planner.network.clone(),
            price_epoch: planner.price_epoch,
            last_feedback: planner.last_feedback,
            generation_policies,
            sources: sources(input),
        }
    }

    pub(crate) const fn complete(&self) -> bool {
        self.complete
    }

    pub(crate) const fn base(&self) -> &AllocationPlan {
        &self.base
    }

    pub(crate) const fn origins(&self) -> &OriginModel {
        &self.origins
    }

    pub(crate) const fn context(&self) -> &PlannerContext {
        &self.context
    }

    pub(crate) const fn config(&self) -> WarpPlannerConfig {
        self.config
    }

    pub(crate) const fn controller_prices(&self) -> ResourcePrices {
        self.controller_prices
    }

    pub(crate) fn network(&self) -> Option<&NetworkTokenBucket> {
        self.network.as_ref()
    }

    pub(crate) const fn price_epoch(&self) -> u64 {
        self.price_epoch
    }

    pub(crate) const fn last_feedback(&self) -> Option<crate::adaptive::ResourceFeedback> {
        self.last_feedback
    }

    pub(crate) const fn generation_policies(&self) -> WarpGenerationPolicies {
        self.generation_policies
    }

    pub(crate) fn sources(&self) -> &[String] {
        &self.sources
    }
}

fn sources(input: &WarpPlannerInput<'_>) -> Vec<String> {
    let mut values = input.snapshot.replay_sources();
    values.extend(input.base.replay_sources());
    values.extend(input.context.replay_sources());
    values.sort();
    values.dedup();
    values
}
