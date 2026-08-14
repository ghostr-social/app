use super::admission::admitted;
use super::allocation::{append_candidate, AppendInputs};
use super::plan::{
    AllocationPlan, AllocationReason, NextReserveEvidence, NextReserveInfeasibility,
    PreemptionAuthority,
};
use super::ranges::{missing, uncovered_bytes};
use super::resources::fully_stored;
use super::sources::best_origin;
use super::{CandidateSnapshot, PlayabilitySnapshot};
use std::collections::HashSet;

pub(super) fn reserve_immediate_next(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    transfer_budget: u64,
    storage_room: u64,
) {
    let Some(candidate) = immediate_next(snapshot) else {
        return;
    };
    if missing(candidate).is_empty() {
        plan.next_reserve = settled_reserve(candidate);
        return;
    }
    let Some(origin) = best_origin(candidate) else {
        return reject(plan, candidate, NextReserveInfeasibility::NoLiveOrigin);
    };
    if !candidate.needs_bootstrap() && !admitted(snapshot, candidate, origin) {
        return reject(plan, candidate, NextReserveInfeasibility::PolicyDenied);
    }
    let budget = transfer_budget.min(storage_room);
    if budget == 0 {
        let reason = zero_budget_reason(storage_room);
        return reject(plan, candidate, reason);
    }
    append_reserve(plan, snapshot, candidate, budget);
}

fn settled_reserve(candidate: &CandidateSnapshot) -> NextReserveEvidence {
    if fully_stored(candidate) {
        return NextReserveEvidence::Ready {
            post: candidate.post.clone(),
        };
    }
    match transfer_covers_remaining(candidate) {
        true => NextReserveEvidence::InFlight {
            post: candidate.post.clone(),
        },
        false => NextReserveEvidence::Infeasible {
            post: candidate.post.clone(),
            reason: NextReserveInfeasibility::PolicyDenied,
        },
    }
}

fn transfer_covers_remaining(candidate: &CandidateSnapshot) -> bool {
    let mut covered = candidate.present.clone();
    covered.extend(
        candidate
            .in_flight
            .iter()
            .filter(|active| active.identity_current)
            .map(|active| active.bytes),
    );
    candidate
        .playable_ranges
        .iter()
        .all(|playable| uncovered_bytes(playable.bytes, &covered) == 0)
}

pub(super) fn reject_unprotected_next(plan: &mut AllocationPlan, snapshot: &PlayabilitySnapshot) {
    if let Some(candidate) = immediate_next(snapshot) {
        reject(
            plan,
            candidate,
            NextReserveInfeasibility::CurrentUnprotected,
        );
    }
}

fn append_reserve(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    budget: u64,
) {
    let first_new = plan.allocations.len();
    let reason = match candidate.needs_bootstrap() {
        true => AllocationReason::MediaBootstrap,
        false => AllocationReason::NextStartability,
    };
    append_candidate(
        plan,
        snapshot,
        AppendInputs {
            candidate,
            target_ms: 1,
            emergency: false,
            budget,
            reason: Some(reason),
        },
    );
    let granted = plan.allocations[first_new..].first();
    plan.next_reserve = match granted {
        Some(work) => NextReserveEvidence::Granted {
            post: candidate.post.clone(),
            range: work.range,
        },
        None => NextReserveEvidence::Infeasible {
            post: candidate.post.clone(),
            reason: NextReserveInfeasibility::NoTransferBudget,
        },
    };
}

fn immediate_next(snapshot: &PlayabilitySnapshot) -> Option<&CandidateSnapshot> {
    snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.feed_offset.value() > 0)
        .min_by_key(|candidate| candidate.feed_offset.magnitude())
}

fn zero_budget_reason(storage_room: u64) -> NextReserveInfeasibility {
    match storage_room {
        0 => NextReserveInfeasibility::NoStorageCapacity,
        _ => NextReserveInfeasibility::NoTransferBudget,
    }
}

fn reject(
    plan: &mut AllocationPlan,
    candidate: &CandidateSnapshot,
    reason: NextReserveInfeasibility,
) {
    plan.next_reserve = NextReserveEvidence::Infeasible {
        post: candidate.post.clone(),
        reason,
    };
}

pub(super) fn planned_bytes(plan: &AllocationPlan) -> u64 {
    plan.allocations.iter().map(|work| work.range.len()).sum()
}

pub(super) fn sibling_planned_bytes(plan: &AllocationPlan, snapshot: &PlayabilitySnapshot) -> u64 {
    plan.allocations
        .iter()
        .filter(|work| work.post != snapshot.playback.current)
        .map(|work| work.range.len())
        .sum()
}

pub(super) fn planned_gain(plan: &AllocationPlan, candidate: &CandidateSnapshot) -> u64 {
    plan.allocations
        .iter()
        .filter(|work| work.post == candidate.post)
        .map(|work| work.expected_playable_gain_ms)
        .sum()
}

pub(super) fn critical_slots(plan: &AllocationPlan) -> usize {
    plan.allocations
        .iter()
        .filter(|work| work.authority == PreemptionAuthority::PlaybackCritical)
        .map(|work| &work.post)
        .collect::<HashSet<_>>()
        .len()
}
