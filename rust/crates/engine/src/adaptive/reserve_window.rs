use super::plan::{AllocationPlan, ControlMode, ReadyReserveEvidence, ReserveCandidateEvidence};
use super::reserve_evidence::{
    count_ordered_ready, count_protected, count_ready, count_structural, immediate_next, initial,
    reject_first_unprepared,
};
use super::reserve_model::{candidates, ready_coverage_ms, target, ReserveTarget};
use super::reserve_schedule::{fill, ScheduleInputs};
use super::PlayabilitySnapshot;

#[derive(Clone, Copy)]
pub(super) enum ReserveModePolicy {
    Ordered,
    LegacyAggregate,
}

#[derive(Clone, Copy)]
pub(super) struct ReserveInputs {
    pub(super) transfer_budget: u64,
    pub(super) storage_room: u64,
    pub(super) current_emergency: bool,
    pub(super) current_protected: bool,
    pub(super) mode_policy: ReserveModePolicy,
}

pub(super) fn build(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    inputs: ReserveInputs,
) {
    let mut window = ReserveWindow::new(snapshot);
    window.prepare(plan, snapshot, inputs);
    window.publish(plan, inputs);
}

struct ReserveWindow<'a> {
    candidates: Vec<&'a super::CandidateSnapshot>,
    target: ReserveTarget,
    evidence: Vec<ReserveCandidateEvidence>,
    counts: InitialCounts,
}

impl<'a> ReserveWindow<'a> {
    fn new(snapshot: &'a PlayabilitySnapshot) -> Self {
        let candidates = candidates(snapshot);
        let target = target(snapshot, &candidates);
        let evidence = initial(&candidates);
        let counts = InitialCounts::from(&evidence);
        Self {
            candidates,
            target,
            evidence,
            counts,
        }
    }

    fn prepare(
        &mut self,
        plan: &mut AllocationPlan,
        snapshot: &PlayabilitySnapshot,
        inputs: ReserveInputs,
    ) {
        if !inputs.current_protected {
            reject_first_unprepared(&mut self.evidence);
            return;
        }
        fill(
            plan,
            &self.candidates,
            &mut self.evidence,
            ScheduleInputs {
                snapshot,
                required: self.target.count,
                origin_candidate_limit: origin_limit(snapshot, inputs.current_emergency),
                transfer_budget: inputs.transfer_budget,
                storage_room: inputs.storage_room,
            },
        );
    }

    fn publish(self, plan: &mut AllocationPlan, inputs: ReserveInputs) {
        plan.mode = self.control_mode(inputs);
        plan.next_reserve = immediate_next(&self.evidence);
        plan.ready_reserve = self.ready_evidence();
    }

    fn control_mode(&self, inputs: ReserveInputs) -> ControlMode {
        match inputs.mode_policy {
            ReserveModePolicy::Ordered => ordered_mode(
                inputs.current_emergency,
                self.counts.ordered_ready,
                self.target.count,
            ),
            ReserveModePolicy::LegacyAggregate => legacy_mode(
                inputs.current_emergency,
                self.counts.ready,
                self.counts.structural,
                self.target.count,
            ),
        }
    }

    fn ready_evidence(self) -> ReadyReserveEvidence {
        ReadyReserveEvidence {
            target: self.target.count,
            ready: self.counts.ready,
            structural: self.counts.structural,
            protected: count_protected(&self.evidence),
            recovery_horizon_ms: self.target.recovery_horizon_ms,
            underflow_risk_bps: self.target.underflow_risk_bps,
            ready_coverage_ms: ready_coverage_ms(&self.candidates, self.target.recovery_horizon_ms),
            candidates: self.evidence,
        }
    }
}

#[derive(Clone, Copy)]
struct InitialCounts {
    ready: usize,
    ordered_ready: usize,
    structural: usize,
}

impl InitialCounts {
    fn from(evidence: &[ReserveCandidateEvidence]) -> Self {
        Self {
            ready: count_ready(evidence),
            ordered_ready: count_ordered_ready(evidence),
            structural: count_structural(evidence),
        }
    }
}

fn origin_limit(snapshot: &PlayabilitySnapshot, emergency: bool) -> Option<usize> {
    let reserve_ceiling = snapshot.network.connection_ceiling.saturating_sub(1).max(1);
    emergency.then_some(
        snapshot
            .network
            .connection_capacity
            .max(2)
            .min(reserve_ceiling),
    )
}

fn ordered_mode(current_emergency: bool, ready: usize, target: usize) -> ControlMode {
    mode(
        current_emergency || (target > 0 && ready == 0),
        ready,
        target,
    )
}

fn legacy_mode(
    current_emergency: bool,
    ready: usize,
    structural: usize,
    target: usize,
) -> ControlMode {
    let empty = target > 0 && ready + structural == 0;
    mode(current_emergency || empty, ready, target)
}

fn mode(emergency: bool, ready: usize, target: usize) -> ControlMode {
    if emergency {
        ControlMode::Emergency
    } else if ready < target {
        ControlMode::Safety
    } else {
        ControlMode::Normal
    }
}
