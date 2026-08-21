use super::{PlanInputs, PlannedWork};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{AdaptivePlayabilityPolicy, PreemptionAuthority};

mod mapping;
mod observability;
mod snapshot;
mod telemetry;
mod warp_context;

pub(super) fn planned_work(
    state: &DeliveryState,
    inputs: PlanInputs<'_>,
    planner: &mut ghostr_engine::adaptive::WarpPlanner,
) -> PlannedWork {
    let Some(snapshot) = snapshot::build(state, &inputs) else {
        return empty_work();
    };
    let allocation = AdaptivePlayabilityPolicy.plan(&snapshot);
    let (context, occupancy) = warp_context::build(state, &snapshot, &allocation, &inputs);
    let warp = planner.plan(ghostr_engine::adaptive::WarpPlannerInput::new(
        &snapshot,
        &allocation,
        inputs.stats.origin_model(),
        &context,
    ));
    let decision_models = observability::models(&snapshot, &inputs, allocation.mode);
    let shadow_prices = observability::shadow_prices(&snapshot, occupancy.total() as u64);
    let emergency = allocation
        .allocations
        .iter()
        .any(|work| work.authority == PreemptionAuthority::PlaybackCritical);
    let transfers = mapping::transfers(state, inputs.present, &allocation);
    let selected_transfers = mapping::selected_transfers(state, inputs.present, &warp);
    let retained = mapping::retained_actions(inputs.in_flight, &warp);
    let evictions = allocation.evictions.clone();
    let discovery_demand = allocation.discovery_demand;
    PlannedWork {
        plan: allocation,
        transfers,
        selected_transfers,
        retained,
        evictions,
        emergency,
        discovery_demand,
        snapshot: Some(snapshot),
        decision_models,
        shadow_prices,
        active_requests: occupancy.total() as u64,
        planner_cpu_micros: 0,
        warp: Some(warp),
    }
}

fn empty_work() -> PlannedWork {
    PlannedWork {
        plan: Default::default(),
        transfers: Vec::new(),
        selected_transfers: Vec::new(),
        retained: Default::default(),
        evictions: Vec::new(),
        emergency: false,
        discovery_demand: ghostr_engine::adaptive::DiscoveryDemand::Expand,
        snapshot: None,
        decision_models: Vec::new(),
        shadow_prices: Default::default(),
        active_requests: 0,
        planner_cpu_micros: 0,
        warp: None,
    }
}
