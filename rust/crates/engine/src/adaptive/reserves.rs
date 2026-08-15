use super::plan::{AllocationPlan, PreemptionAuthority};
use super::{CandidateSnapshot, PlayabilitySnapshot};
use std::collections::HashSet;

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
