use super::plan::{AllocationPlan, ControlMode, ReadyReserveEvidence};
use super::reserve_evidence::{
    count_protected, count_ready, count_structural, immediate_next, initial,
    reject_first_unprepared,
};
use super::reserve_model::{candidates, ready_coverage_ms, target};
use super::reserve_schedule::{fill, ScheduleInputs};
use super::PlayabilitySnapshot;

pub(super) struct ReserveInputs {
    pub(super) transfer_budget: u64,
    pub(super) storage_room: u64,
    pub(super) current_emergency: bool,
    pub(super) current_protected: bool,
}

pub(super) fn build(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    inputs: ReserveInputs,
) {
    let candidates = candidates(snapshot);
    let target = target(snapshot, &candidates);
    let mut evidence = initial(&candidates);
    let ready = count_ready(&evidence);
    let structural = count_structural(&evidence);
    let required = target.count;
    if inputs.current_protected {
        fill(
            plan,
            &candidates,
            &mut evidence,
            ScheduleInputs {
                snapshot,
                required,
                origin_candidate_limit: origin_limit(snapshot, inputs.current_emergency),
                transfer_budget: inputs.transfer_budget,
                storage_room: inputs.storage_room,
            },
        );
    } else {
        reject_first_unprepared(&mut evidence);
    }
    plan.mode = mode(inputs.current_emergency, ready, structural, target.count);
    plan.next_reserve = immediate_next(&evidence);
    plan.ready_reserve = ReadyReserveEvidence {
        target: target.count,
        ready,
        structural,
        protected: count_protected(&evidence),
        recovery_horizon_ms: target.recovery_horizon_ms,
        underflow_risk_bps: target.underflow_risk_bps,
        ready_coverage_ms: ready_coverage_ms(&candidates, target.recovery_horizon_ms),
        candidates: evidence,
    };
}

fn origin_limit(snapshot: &PlayabilitySnapshot, emergency: bool) -> Option<usize> {
    emergency.then_some(snapshot.network.connection_capacity.max(1))
}

fn mode(current_emergency: bool, ready: usize, structural: usize, target: usize) -> ControlMode {
    if current_emergency || (target > 0 && ready + structural == 0) {
        ControlMode::Emergency
    } else if ready < target {
        ControlMode::Safety
    } else {
        ControlMode::Normal
    }
}
