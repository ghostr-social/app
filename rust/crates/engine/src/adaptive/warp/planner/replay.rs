use super::{WarpPlanner, WarpPlannerConfig, WarpPlannerInput, WarpPlanningDecision};
use crate::adaptive::{
    AllocationPlan, HlsGenerationPolicy, NetworkTokenBucket, PlannerContext, ResourcePrices,
    ShadowPriceController,
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
    hls_generation_policy: HlsGenerationPolicy,
    sources: Vec<String>,
}

impl PlannerReplayCapsule {
    pub(super) fn capture(
        input: &WarpPlannerInput<'_>,
        planner: &WarpPlanner,
        hls_generation_policy: HlsGenerationPolicy,
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
            hls_generation_policy,
            sources: sources(input),
        }
    }

    pub(crate) fn run(
        &self,
        snapshot: &crate::adaptive::PlayabilitySnapshot,
    ) -> WarpPlanningDecision {
        let mut planner = WarpPlanner {
            config: self.config,
            twin: crate::adaptive::DigitalTwin::new(self.config.twin),
            prices: ShadowPriceController::from_prices(self.controller_prices),
            network: self.network.clone(),
            price_epoch: self.price_epoch,
            last_feedback: self.last_feedback,
        };
        planner.plan_with_hls_policy(
            WarpPlannerInput::new(snapshot, &self.base, &self.origins, &self.context),
            self.hls_generation_policy,
        )
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

    pub(crate) const fn hls_generation_policy(&self) -> HlsGenerationPolicy {
        self.hls_generation_policy
    }

    pub(crate) fn sources(&self) -> &[String] {
        &self.sources
    }

    pub(crate) fn restored(
        base: AllocationPlan,
        origins: OriginModel,
        context: PlannerContext,
        state: PlannerReplayState,
        sources: Vec<String>,
    ) -> Self {
        Self {
            complete: true,
            base,
            origins,
            context,
            config: state.config,
            controller_prices: state.controller_prices,
            network: state.network,
            price_epoch: state.price_epoch,
            last_feedback: state.last_feedback,
            hls_generation_policy: state.hls_generation_policy,
            sources,
        }
    }
}

pub(crate) struct PlannerReplayState {
    pub config: WarpPlannerConfig,
    pub controller_prices: ResourcePrices,
    pub network: Option<NetworkTokenBucket>,
    pub price_epoch: u64,
    pub last_feedback: Option<crate::adaptive::ResourceFeedback>,
    pub hls_generation_policy: HlsGenerationPolicy,
}

fn sources(input: &WarpPlannerInput<'_>) -> Vec<String> {
    let mut values = input.snapshot.replay_sources();
    values.extend(input.base.replay_sources());
    values.extend(input.context.replay_sources());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
#[path = "replay_axiom_test.rs"]
pub(crate) mod axiom_test_support;
