mod capacity_demand;
mod feasibility;
mod least_risk;
mod replay;
mod reserve;
mod reset;
mod search_replay;
mod search_run;
mod simulation;
mod types;

pub(crate) use replay::{PlannerReplayCapsule, PlannerReplayState};
pub use reserve::{
    RescueChanceEvidence, RescueTimingQuantile, ReserveAuthorityOccupancy, ReserveConstraint,
    ReserveDegradedReason,
};
pub(crate) use search_replay::{SearchReplayInput, SearchReplayMode};
pub use types::{SemanticDecision, WarpPlannerConfig, WarpPlannerInput, WarpPlanningDecision};

use super::{
    ActionFrontier, DigitalTwin, GeneratedAction, NetworkTokenBucket, ResourceCost,
    ShadowPriceController, WarpActionGenerator,
};

pub struct WarpPlanner {
    config: WarpPlannerConfig,
    twin: DigitalTwin,
    prices: ShadowPriceController,
    network: Option<NetworkTokenBucket>,
    price_epoch: u64,
}

impl WarpPlanner {
    pub fn new(config: WarpPlannerConfig) -> Self {
        Self {
            twin: DigitalTwin::new(config.twin),
            config,
            prices: ShadowPriceController::default(),
            network: None,
            price_epoch: 0,
        }
    }

    pub fn plan(&mut self, input: WarpPlannerInput<'_>) -> WarpPlanningDecision {
        let planner_replay = PlannerReplayCapsule::capture(&input, self);
        self.observe_prices(&input);
        self.prepare_network(&input);
        let generated =
            WarpActionGenerator::generate(input.snapshot, input.base, input.origins, input.context);
        let frontier = ActionFrontier::prune(
            generated
                .actions
                .iter()
                .map(|item| item.node.clone())
                .collect(),
            input.context.epsilon,
        );
        let network_bytes = self.network_tokens(input.snapshot.observed_at_ms);
        let feasible = feasibility::apply(&input, &frontier.retained, &self.config, network_bytes);
        let (search, search_replay) = self.search(&input, &feasible);
        let additional_request_slot_demanded =
            self.additional_request_slot_demanded(&input, &frontier.retained, network_bytes);
        let selected = selected_action(&generated.actions, &search);
        let common_random_seed = simulation::common_seed(&input, self.price_epoch);
        let evaluation = selected.as_ref().map(|item| {
            self.twin.evaluate(
                &simulation::state(&input),
                std::slice::from_ref(&item.node),
                simulation::epochs(&input, self.price_epoch),
            )
        });
        WarpPlanningDecision {
            pruned_action_ids: pruned_ids(&generated.actions, &feasible.nodes),
            admissible_action_ids: feasible.nodes.iter().map(|node| node.id).collect(),
            selected,
            search,
            evaluation,
            reserve: feasible.reserve,
            semantic: feasible.semantic,
            prices: self.prices.prices(),
            additional_request_slot_demanded,
            common_random_seed,
            retry_availability: input.context.retry_evidence(input.snapshot),
            search_replay: Some(search_replay),
            planner_replay: Some(planner_replay),
            generated,
        }
    }

    /// Charges the authorized network envelope before execution starts.
    pub fn commit(
        &mut self,
        action: &GeneratedAction,
        committed: ResourceCost,
        observed_at_ms: u64,
    ) -> bool {
        if !committed.no_more_than(action.node.authorized_resources()) {
            return false;
        }
        let Some(bucket) = self.network.as_mut() else {
            return true;
        };
        let mut updated = bucket.clone();
        if !updated.consume(committed.network_bytes, observed_at_ms) {
            return false;
        }
        *bucket = updated;
        true
    }

    pub fn reconcile_network_reservation(
        &mut self,
        reserved_bytes: u64,
        actual_bytes: u64,
        observed_at_ms: u64,
    ) {
        if let Some(bucket) = self.network.as_mut() {
            bucket.reconcile_reservation(reserved_bytes, actual_bytes, observed_at_ms);
        }
    }

    pub fn network_tokens(&mut self, observed_at_ms: u64) -> u64 {
        self.network
            .as_mut()
            .map_or(0, |bucket| bucket.available(observed_at_ms))
    }

    pub fn network_refill_deadline_ms(
        &mut self,
        required_bytes: u64,
        observed_at_ms: u64,
    ) -> Option<u64> {
        self.network
            .as_mut()?
            .refill_deadline_ms(required_bytes, observed_at_ms)
    }

    pub fn next_network_refill_deadline_ms(
        &mut self,
        actions: &[GeneratedAction],
        observed_at_ms: u64,
    ) -> Option<u64> {
        actions
            .iter()
            .map(|action| action.node.authorized_resources().network_bytes)
            .filter(|bytes| *bytes > 0)
            .filter_map(|bytes| self.network_refill_deadline_ms(bytes, observed_at_ms))
            .min()
    }

    fn observe_prices(&mut self, input: &WarpPlannerInput<'_>) {
        let before = self.prices.prices();
        if let Some(feedback) = input.context.feedback {
            self.prices.observe(feedback.actual, feedback.target);
        }
        if self.prices.prices() != before {
            self.price_epoch = self.price_epoch.saturating_add(1);
        }
        self.twin.set_prices(self.prices.prices());
    }

    fn prepare_network(&mut self, input: &WarpPlannerInput<'_>) {
        let limits = input.context.limits;
        match &mut self.network {
            Some(bucket) => bucket.reconfigure(
                limits.network_burst_bytes,
                limits.network_rate_bytes_per_second,
                input.snapshot.observed_at_ms,
            ),
            None => {
                self.network = Some(NetworkTokenBucket::new(
                    limits.network_burst_bytes,
                    limits.network_rate_bytes_per_second,
                    input.snapshot.observed_at_ms,
                ));
            }
        }
    }
}
impl Default for WarpPlanner {
    fn default() -> Self {
        Self::new(WarpPlannerConfig::default())
    }
}

fn selected_action(
    generated: &[GeneratedAction],
    search: &super::SearchDecision,
) -> Option<GeneratedAction> {
    let id = search.action.as_ref()?.id;
    generated.iter().find(|item| item.node.id == id).cloned()
}

fn pruned_ids(generated: &[GeneratedAction], retained: &[super::ActionNode]) -> Vec<u16> {
    generated
        .iter()
        .filter(|item| !retained.iter().any(|node| node.id == item.node.id))
        .map(|item| item.node.id)
        .collect()
}
