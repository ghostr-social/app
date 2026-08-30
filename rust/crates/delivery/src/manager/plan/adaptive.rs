use super::{PlanInputs, PlannedWork};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    AdaptivePlayabilityPolicy, PreemptionAuthority, WarpPlanningDecision,
};

mod mapping;
mod observability;
mod snapshot;
mod telemetry;
mod warp_context;
mod watch;

pub(super) fn planned_work(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    planner: &mut ghostr_engine::adaptive::WarpPlanner,
    model: &ghostr_engine::watch_model::WatchModel,
) -> PlannedWork {
    let Some(mut snapshot) = snapshot::build(state, inputs) else {
        return empty_work();
    };
    let watch = watch::WatchPlanningWindow::predict(&mut snapshot, model);
    let allocation = AdaptivePlayabilityPolicy.plan(&snapshot);
    let (context, occupancy, hedge_tails) =
        warp_context::build(state, &snapshot, &allocation, inputs);
    let context = watch.apply_context(context);
    let warp = planner.plan(ghostr_engine::adaptive::WarpPlannerInput::new(
        &snapshot,
        &allocation,
        inputs.stats.origin_model(),
        &context,
    ));
    let network_refill_deadline_ms =
        network_refill_deadline(planner, &warp, snapshot.observed_at_ms);
    let decision_models = observability::models(&snapshot, inputs, allocation.mode);
    let shadow_prices = observability::shadow_prices(&snapshot, occupancy.total() as u64);
    let emergency = has_playback_critical_work(&allocation);
    let transfers = Vec::new();
    let selected_transfers =
        mapping::selected_transfers(state, inputs.present, &warp, allocation.mode);
    let retained = mapping::retained_actions(inputs.in_flight, &warp);
    let retained_posts = mapping::retained_posts(inputs.in_flight, &retained);
    let evictions = allocation.evictions.clone();
    let discovery_demand = allocation.discovery_demand;
    PlannedWork {
        plan: allocation,
        transfers,
        selected_transfers,
        retained,
        retained_posts,
        evictions,
        emergency,
        discovery_demand,
        snapshot: Some(snapshot),
        decision_models,
        shadow_prices,
        active_requests: occupancy.total() as u64,
        hedge_tails,
        network_refill_deadline_ms,
        planner_cpu_micros: 0,
        warp: Some(warp),
        player_preparations: Vec::new(),
    }
}

fn has_playback_critical_work(allocation: &ghostr_engine::adaptive::AllocationPlan) -> bool {
    allocation
        .allocations
        .iter()
        .any(|work| work.authority == PreemptionAuthority::PlaybackCritical)
}

fn empty_work() -> PlannedWork {
    PlannedWork {
        plan: Default::default(),
        transfers: Vec::new(),
        selected_transfers: Vec::new(),
        retained: Default::default(),
        retained_posts: Default::default(),
        evictions: Vec::new(),
        emergency: false,
        discovery_demand: ghostr_engine::adaptive::DiscoveryDemand::Expand,
        snapshot: None,
        decision_models: Vec::new(),
        shadow_prices: Default::default(),
        active_requests: 0,
        hedge_tails: Vec::new(),
        network_refill_deadline_ms: None,
        planner_cpu_micros: 0,
        warp: None,
        player_preparations: Vec::new(),
    }
}

fn network_refill_deadline(
    planner: &mut ghostr_engine::adaptive::WarpPlanner,
    decision: &WarpPlanningDecision,
    observed_at_ms: u64,
) -> Option<u64> {
    planner.next_network_refill_deadline_ms(&decision.generated.actions, observed_at_ms)
}
