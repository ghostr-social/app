use super::{WarpPlanner, WarpPlannerConfig, WarpPlannerInput, WarpPlanningDecision};
use crate::adaptive::{
    AllocationPlan, NetworkTokenBucket, PlannerContext, ResourcePrices, ShadowPriceController,
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
    sources: Vec<String>,
}

impl PlannerReplayCapsule {
    pub(super) fn capture(input: &WarpPlannerInput<'_>, planner: &WarpPlanner) -> Self {
        Self {
            complete: true,
            base: input.base.clone(),
            origins: input.origins.clone(),
            context: input.context.clone(),
            config: planner.config,
            controller_prices: planner.prices.prices(),
            network: planner.network.clone(),
            price_epoch: planner.price_epoch,
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
        };
        planner.plan(WarpPlannerInput::new(
            snapshot,
            &self.base,
            &self.origins,
            &self.context,
        ))
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

    pub(crate) fn sources(&self) -> &[String] {
        &self.sources
    }

    #[cfg(test)]
    pub(crate) fn mark_incomplete(&mut self) {
        self.complete = false;
    }

    #[cfg(test)]
    pub(crate) fn base_mut(&mut self) -> &mut AllocationPlan {
        &mut self.base
    }

    #[cfg(test)]
    pub(crate) fn origins_mut(&mut self) -> &mut OriginModel {
        &mut self.origins
    }

    #[cfg(test)]
    pub(crate) fn context_mut(&mut self) -> &mut PlannerContext {
        &mut self.context
    }

    #[cfg(test)]
    pub(crate) fn config_mut(&mut self) -> &mut WarpPlannerConfig {
        &mut self.config
    }

    #[cfg(test)]
    pub(crate) fn controller_prices_mut(&mut self) -> &mut ResourcePrices {
        &mut self.controller_prices
    }

    #[cfg(test)]
    pub(crate) fn network_mut(&mut self) -> Option<&mut NetworkTokenBucket> {
        self.network.as_mut()
    }

    #[cfg(test)]
    pub(crate) fn price_epoch_mut(&mut self) -> &mut u64 {
        &mut self.price_epoch
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
            sources,
        }
    }
}

pub(crate) struct PlannerReplayState {
    pub config: WarpPlannerConfig,
    pub controller_prices: ResourcePrices,
    pub network: Option<NetworkTokenBucket>,
    pub price_epoch: u64,
}

fn sources(input: &WarpPlannerInput<'_>) -> Vec<String> {
    let mut values = input.snapshot.replay_sources();
    values.extend(input.base.replay_sources());
    values.extend(input.context.replay_sources());
    values.sort();
    values.dedup();
    values
}
