use super::{PlanInputs, PlannedWork};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{AdaptivePlayabilityPolicy, PreemptionAuthority};

mod mapping;
mod snapshot;
mod telemetry;

pub(super) fn planned_work(state: &DeliveryState, inputs: PlanInputs<'_>) -> PlannedWork {
    let Some(snapshot) = snapshot::build(state, &inputs) else {
        return empty_work();
    };
    let allocation = AdaptivePlayabilityPolicy.plan(&snapshot);
    let emergency = allocation
        .allocations
        .iter()
        .any(|work| work.authority == PreemptionAuthority::PlaybackCritical);
    let transfers = mapping::transfers(state, inputs.present, &allocation);
    let retained = mapping::retained_transfers(state, &allocation);
    let evictions = allocation.evictions.clone();
    let discovery_demand = allocation.discovery_demand;
    PlannedWork {
        plan: allocation,
        transfers,
        retained,
        evictions,
        emergency,
        discovery_demand,
    }
}

fn empty_work() -> PlannedWork {
    PlannedWork {
        plan: Default::default(),
        transfers: Vec::new(),
        retained: Default::default(),
        evictions: Vec::new(),
        emergency: false,
        discovery_demand: ghostr_engine::adaptive::DiscoveryDemand::Expand,
    }
}
