use super::reserves::sibling_planned_bytes;
use super::sources::candidate_score;
use super::{AllocationPlan, CandidateSnapshot, DiscoveryDemand, PlayabilitySnapshot};
use std::collections::HashSet;

pub(super) fn discovery_demand(
    snapshot: &PlayabilitySnapshot,
    plan: &AllocationPlan,
    speculative_budget: u64,
) -> DiscoveryDemand {
    if speculative_budget <= sibling_planned_bytes(plan, snapshot) {
        return DiscoveryDemand::Hold;
    }
    let admitted: HashSet<_> = plan
        .allocations
        .iter()
        .map(|work| &work.post)
        .chain(plan.retained.iter().map(|work| &work.post))
        .collect();
    match has_waiting_candidate(snapshot, &admitted) {
        true => DiscoveryDemand::Hold,
        false => DiscoveryDemand::Expand,
    }
}

fn has_waiting_candidate(
    snapshot: &PlayabilitySnapshot,
    admitted: &HashSet<&crate::PostId>,
) -> bool {
    snapshot.candidates.iter().any(|candidate| {
        candidate.post != snapshot.playback.current
            && !super::ranges::missing(candidate).is_empty()
            && !admitted.contains(&candidate.post)
    })
}

pub(super) fn upcoming_candidates(snapshot: &PlayabilitySnapshot) -> Vec<&CandidateSnapshot> {
    let mut candidates: Vec<_> = snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.post != snapshot.playback.current)
        .collect();
    candidates.sort_by(|left, right| {
        candidate_score(snapshot, right)
            .total_cmp(&candidate_score(snapshot, left))
            .then_with(|| {
                left.feed_offset
                    .magnitude()
                    .cmp(&right.feed_offset.magnitude())
            })
    });
    candidates
}
