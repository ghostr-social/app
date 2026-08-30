use crate::manager::plan::axiom_test_support::planned_work;
use crate::manager::plan::{PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{PlannerCommand, StorageSnapshot};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};

#[path = "warp_head_probe_context_fixture/state.rs"]
mod state_fixture;
pub(super) use state_fixture::{
    ahead_state, ahead_state_with_size, ahead_state_with_sources, current_state,
};

pub(super) fn generates_head_for(work: PlannedWork, post: &PostId) -> bool {
    work.warp.expect("valid test fixture").generated.actions.iter().any(|action| {
        matches!(&action.command, PlannerCommand::ProbeHead { post: candidate, .. } if candidate == post)
    })
}

pub(super) fn plan(
    state: &DeliveryState,
    active: &[TransferIdentity],
    capacity: usize,
) -> PlannedWork {
    plan_at(state, active, &HashSet::new(), 1, capacity)
}

pub(super) fn plan_at(
    state: &DeliveryState,
    active: &[TransferIdentity],
    completed: &HashSet<TransferIdentity>,
    observed_at_ms: u64,
    capacity: usize,
) -> PlannedWork {
    planned_work(
        state,
        &PlanInputs {
            stats: &HostStats::new(),
            retry: &RetryBook::new(RetryPolicy::default()),
            present: &HashMap::new(),
            finalized: &HashSet::new(),
            stored_totals: &HashMap::new(),
            continuation_sources: &HashMap::new(),
            revisions: &HashMap::new(),
            independent_sources: &HashMap::new(),
            whole_body_exhaustions: &HashMap::new(),
            completed_head_probes: completed,
            unavailable_head_probes: &HashSet::new(),
            in_flight: &[],
            active_head_probes: active,
            hls_candidates: &[],
            active_hls_sources: &[],
            segmented_storage_available_bytes: u64::MAX,
            storage: StorageSnapshot::new(1_000_000, 0),
            connection_capacity: capacity,
            hls_demand_expansion_allowed: true,
            connection_ceiling: 3,
            per_authority_request_limit: 3,
            packet_loss_bps: 0,
            resource_feedback: None,
            capacity_revision: 0,
            observed_at_ms,
            demanded: &HashMap::new(),
        },
    )
}
