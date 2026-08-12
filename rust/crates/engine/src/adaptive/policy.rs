use super::allocation::{append_candidate, planned_bytes, AppendInputs};
use super::commitments::retained;
use super::eviction::evictions;
use super::plan::{AllocationPlan, DiscoveryDemand, PreemptionAuthority};
use super::ranges::missing;
use super::resources::{endangered, speculative_budget, upcoming_depth_ms};
use super::sources::candidate_score;
use super::{CandidateSnapshot, PlayabilitySnapshot};
use std::collections::HashSet;

const EMERGENCY_DEPTH_MS: u64 = 12_000;
const EMERGENCY_TRANSITION_DEPTH_MS: u64 = 2_000;
const SAFE_CURRENT_DEPTH_MS: u64 = 2_000;

#[derive(Clone, Copy, Debug, Default)]
pub struct AdaptivePlayabilityPolicy;

impl AdaptivePlayabilityPolicy {
    pub fn plan(self, snapshot: &PlayabilitySnapshot) -> AllocationPlan {
        let Some(current) = current_candidate(snapshot) else {
            return AllocationPlan {
                discovery_demand: DiscoveryDemand::Expand,
                ..AllocationPlan::default()
            };
        };
        let emergency = endangered(snapshot, current);
        let mut plan = initial_plan(snapshot);
        append_candidate(&mut plan, snapshot, current_append(current, emergency));
        plan.retained = retained(snapshot, emergency, critical_slots(&plan));
        if emergency {
            if !current_is_protected(&plan, current) {
                return plan;
            }
            append_emergency_transition(&mut plan, snapshot);
            return plan;
        }
        append_upcoming(&mut plan, snapshot);
        plan.discovery_demand = discovery_demand(snapshot, &plan);
        plan
    }
}

fn current_is_protected(plan: &AllocationPlan, current: &CandidateSnapshot) -> bool {
    missing(current).is_empty()
        || plan
            .allocations
            .iter()
            .any(|work| work.post == current.post)
        || plan.retained.iter().any(|work| work.post == current.post)
}

fn discovery_demand(snapshot: &PlayabilitySnapshot, plan: &AllocationPlan) -> DiscoveryDemand {
    if speculative_budget(snapshot) <= planned_bytes(plan) {
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
            && !missing(candidate).is_empty()
            && !admitted.contains(&candidate.post)
    })
}

fn initial_plan(snapshot: &PlayabilitySnapshot) -> AllocationPlan {
    AllocationPlan {
        evictions: evictions(snapshot),
        ..AllocationPlan::default()
    }
}

fn critical_slots(plan: &AllocationPlan) -> usize {
    plan.allocations
        .iter()
        .filter(|work| work.authority == PreemptionAuthority::PlaybackCritical)
        .map(|work| &work.post)
        .collect::<HashSet<_>>()
        .len()
}

fn append_emergency_transition(plan: &mut AllocationPlan, snapshot: &PlayabilitySnapshot) {
    let budget = speculative_budget(snapshot).saturating_sub(planned_bytes(plan));
    let Some(candidate) = upcoming_candidates(snapshot).into_iter().next() else {
        return;
    };
    append_candidate(
        plan,
        snapshot,
        AppendInputs {
            candidate,
            target_ms: EMERGENCY_TRANSITION_DEPTH_MS,
            emergency: false,
            budget,
        },
    );
}

fn current_candidate(snapshot: &PlayabilitySnapshot) -> Option<&CandidateSnapshot> {
    snapshot
        .candidates
        .iter()
        .find(|candidate| candidate.post == snapshot.playback.current)
}

fn current_append(candidate: &CandidateSnapshot, emergency: bool) -> AppendInputs<'_> {
    AppendInputs {
        candidate,
        target_ms: match emergency {
            true => EMERGENCY_DEPTH_MS,
            false => SAFE_CURRENT_DEPTH_MS,
        },
        emergency,
        budget: u64::MAX,
    }
}

fn append_upcoming(plan: &mut AllocationPlan, snapshot: &PlayabilitySnapshot) {
    let mut budget = speculative_budget(snapshot).saturating_sub(planned_bytes(plan));
    for candidate in upcoming_candidates(snapshot) {
        let inputs = AppendInputs {
            candidate,
            target_ms: upcoming_depth_ms(snapshot),
            emergency: false,
            budget,
        };
        budget = append_candidate(plan, snapshot, inputs);
        if budget == 0 {
            return;
        }
    }
}

fn upcoming_candidates(snapshot: &PlayabilitySnapshot) -> Vec<&CandidateSnapshot> {
    let mut candidates: Vec<_> = snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.post != snapshot.playback.current)
        .collect();
    candidates.sort_by(|left, right| {
        candidate_score(snapshot, right)
            .total_cmp(&candidate_score(snapshot, left))
            .then_with(|| left.feed_distance.cmp(&right.feed_distance))
    });
    candidates
}
