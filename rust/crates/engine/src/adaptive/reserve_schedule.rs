use super::admission::admitted;
use super::allocation::{append_candidate, AppendInputs};
use super::plan::{
    AllocationPlan, AllocationReason, NextReserveInfeasibility, ReserveCandidateEvidence,
    ReserveCandidateState,
};
use super::reserve_model::readiness_ranges;
use super::reserve_origins::OriginSlots;
use super::reserves::{planned_bytes, sibling_planned_bytes};
use super::sources::best_origin;
use super::{CandidateSnapshot, MediaLayout, PlayabilitySnapshot};

pub(super) struct ScheduleInputs<'a> {
    pub(super) snapshot: &'a PlayabilitySnapshot,
    pub(super) required: usize,
    pub(super) origin_candidate_limit: Option<usize>,
    pub(super) transfer_budget: u64,
    pub(super) storage_room: u64,
}

pub(super) fn fill(
    plan: &mut AllocationPlan,
    candidates: &[&CandidateSnapshot],
    evidence: &mut [ReserveCandidateEvidence],
    inputs: ScheduleInputs<'_>,
) {
    let mut protected = 0;
    let mut origins = OriginSlots::new(candidates, inputs.origin_candidate_limit);
    for (candidate, item) in candidates.iter().zip(evidence) {
        if protected >= inputs.required {
            return;
        }
        if super::reserve_evidence::is_protected_state(&item.state) {
            protected += 1;
            continue;
        }
        if item.state != ReserveCandidateState::Unprepared || !origins.available(candidate) {
            continue;
        }
        let allocations = plan.allocations.len();
        let outcome = schedule_candidate(plan, candidate, &inputs);
        if plan.allocations.len() > allocations {
            origins.occupy(candidate);
        }
        protected += usize::from(outcome.protected);
        item.state = outcome.state;
    }
}

struct ScheduleOutcome {
    state: ReserveCandidateState,
    protected: bool,
}

fn schedule_candidate(
    plan: &mut AllocationPlan,
    candidate: &CandidateSnapshot,
    inputs: &ScheduleInputs<'_>,
) -> ScheduleOutcome {
    let Some(origin) = best_origin(candidate) else {
        return unavailable(NextReserveInfeasibility::NoLiveOrigin);
    };
    if !candidate.needs_bootstrap() && !admitted(inputs.snapshot, candidate, origin) {
        return unavailable(NextReserveInfeasibility::PolicyDenied);
    }
    let transfer = inputs
        .transfer_budget
        .saturating_sub(sibling_planned_bytes(plan, inputs.snapshot));
    let storage = inputs.storage_room.saturating_sub(planned_bytes(plan));
    let required = readiness_bytes(candidate);
    if required == 0 {
        return probing(candidate);
    }
    let available = transfer.min(storage);
    let budget = readiness_budget(candidate, required, available);
    if budget == 0 {
        let reason = match storage {
            0 => NextReserveInfeasibility::NoStorageCapacity,
            _ => NextReserveInfeasibility::NoTransferBudget,
        };
        return unavailable(reason);
    }
    append_readiness(plan, inputs.snapshot, candidate, budget);
    planned_outcome(plan, candidate)
}

fn readiness_budget(candidate: &CandidateSnapshot, required: u64, available: u64) -> u64 {
    match candidate.layout {
        MediaLayout::RequiresCompleteFile if required > available => 0,
        MediaLayout::RequiresCompleteFile => required,
        _ => available,
    }
}

fn append_readiness(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    budget: u64,
) {
    let reason = match candidate.needs_bootstrap() {
        true => AllocationReason::MediaBootstrap,
        false => AllocationReason::NextStartability,
    };
    append_candidate(
        plan,
        snapshot,
        AppendInputs {
            candidate,
            target_ms: first_playable_ms(candidate),
            emergency: false,
            budget,
            reason: Some(reason),
        },
    );
}

fn planned_outcome(plan: &AllocationPlan, candidate: &CandidateSnapshot) -> ScheduleOutcome {
    let ranges: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == candidate.post)
        .map(|work| work.request.requested_bytes())
        .collect();
    if ranges.is_empty() {
        return unavailable(NextReserveInfeasibility::NoTransferBudget);
    }
    let protected = planned_covers(candidate, &ranges);
    let state = match protected {
        true => ReserveCandidateState::Planned { ranges },
        false => ReserveCandidateState::Preparing { ranges },
    };
    ScheduleOutcome { protected, state }
}

fn planned_covers(candidate: &CandidateSnapshot, planned: &[crate::ByteRange]) -> bool {
    if candidate.startup.is_none() {
        return false;
    }
    readiness_ranges(candidate)
        .iter()
        .all(|range| uncovered(candidate, planned, *range) == 0)
}

fn readiness_bytes(candidate: &CandidateSnapshot) -> u64 {
    readiness_ranges(candidate)
        .iter()
        .map(|range| uncovered(candidate, &[], *range))
        .sum()
}

fn uncovered(
    candidate: &CandidateSnapshot,
    planned: &[crate::ByteRange],
    range: crate::ByteRange,
) -> u64 {
    let mut covered = candidate.present.clone();
    covered.extend(
        candidate
            .in_flight
            .iter()
            .filter(|active| active.identity_current)
            .map(|active| active.effective_bytes),
    );
    covered.extend_from_slice(planned);
    super::ranges::uncovered_bytes(range, &covered)
}

fn first_playable_ms(candidate: &CandidateSnapshot) -> u64 {
    candidate
        .playable_ranges
        .first()
        .map_or(1, |playable| playable.playable_ms.max(1))
}

fn probing(candidate: &CandidateSnapshot) -> ScheduleOutcome {
    let state = match candidate.layout {
        MediaLayout::Unknown => ReserveCandidateState::Probing,
        _ => ReserveCandidateState::Unprepared,
    };
    ScheduleOutcome {
        state,
        protected: false,
    }
}

fn unavailable(reason: NextReserveInfeasibility) -> ScheduleOutcome {
    ScheduleOutcome {
        state: ReserveCandidateState::Infeasible { reason },
        protected: false,
    }
}
