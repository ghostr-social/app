use super::DecisionResolution;
use crate::delivery_events::CommandReceiver;
use ghostr_engine::adaptive::{
    AllocationPlan, DecisionModelInput, DecisionOutcome, PlayabilitySnapshot, ShadowPrices,
};
use ghostr_engine::ActionId;

impl CommandReceiver {
    pub(crate) fn publish_decision(
        &self,
        snapshot: &PlayabilitySnapshot,
        plan: &AllocationPlan,
        prices: ShadowPrices,
        models: &[DecisionModelInput],
    ) -> u64 {
        self.decisions.publish(snapshot, plan, prices, models)
    }

    pub(crate) fn bind_latest_decision(&self, action: ActionId, observed_at_ms: u64) -> bool {
        self.decisions.bind_latest(action, observed_at_ms)
    }

    pub(crate) fn resolve_decision(
        &self,
        action: ActionId,
        outcome: DecisionOutcome,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        self.decisions.resolve(action, outcome, observed_at_ms)
    }

    pub(crate) fn resolve_latest_decision(&self, outcome: DecisionOutcome) -> bool {
        self.decisions.resolve_latest(outcome)
    }
}
