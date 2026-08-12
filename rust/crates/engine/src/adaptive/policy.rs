use super::allocation::{append_candidate, planned_bytes, AppendInputs};
use super::commitments::retained;
use super::eviction::evictions;
use super::plan::{AllocationPlan, DiscoveryDemand, PreemptionAuthority};
use super::ranges::{missing, playable_gain};
use super::resources::{endangered, speculative_budget, upcoming_depth_ms};
use super::sources::candidate_score;
use super::{CandidateSnapshot, PlayabilitySnapshot};
use std::collections::HashSet;

const EMERGENCY_DEPTH_MS: u64 = 12_000;
const EMERGENCY_TRANSITION_DEPTH_MS: u64 = 2_000;
/// Safe-mode planning depth for the current video. Must sit well above
/// the 4-second emergency threshold in [`super::resources::endangered`],
/// or steady playback oscillates across that line and every crossing
/// cancels speculative work — but not so deep that the current video's
/// reserve starves the next video's head on a constrained link.
const SAFE_CURRENT_DEPTH_MS: u64 = 6_000;

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
        let mut plan = initial_plan(snapshot, current_inflight_need_bytes(current));
        let storage_room = current_storage_room(snapshot, &plan);
        append_candidate(
            &mut plan,
            snapshot,
            current_append(snapshot, current, emergency, storage_room),
        );
        plan.retained = retained(snapshot, emergency, critical_slots(&plan));
        if emergency {
            if current_is_protected(&plan, current) {
                append_emergency_transition(&mut plan, snapshot);
            }
            plan.discovery_demand = discovery_demand(snapshot, &plan);
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

/// The current video is exempt from the speculative budget, so only
/// sibling allocations count against it here; otherwise any real-sized
/// current download would idle feed discovery for its whole duration.
fn discovery_demand(snapshot: &PlayabilitySnapshot, plan: &AllocationPlan) -> DiscoveryDemand {
    let speculative: u64 = plan
        .allocations
        .iter()
        .filter(|work| work.post != snapshot.playback.current)
        .map(|work| work.range.len())
        .sum();
    if speculative_budget(snapshot) <= speculative {
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

fn initial_plan(snapshot: &PlayabilitySnapshot, current_need_bytes: u64) -> AllocationPlan {
    AllocationPlan {
        evictions: evictions(snapshot, current_need_bytes),
        ..AllocationPlan::default()
    }
}

/// Current-video bytes already on the wire will land shortly; the
/// eviction pass makes hard room for exactly those so the store never
/// has to refuse them. Bytes that are merely planned make no room
/// claim — their requests are budgeted to the room that exists.
fn current_inflight_need_bytes(candidate: &CandidateSnapshot) -> u64 {
    candidate
        .in_flight
        .iter()
        .filter(|active| active.identity_current)
        .map(|active| super::ranges::uncovered_bytes(active.bytes, &candidate.present))
        .sum()
}

fn current_target_ms(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    emergency: bool,
) -> u64 {
    let depth = match emergency {
        true => EMERGENCY_DEPTH_MS,
        false => SAFE_CURRENT_DEPTH_MS,
    };
    let covered = snapshot
        .playback
        .buffer_ahead_ms
        .saturating_add(inflight_playable_ms(candidate));
    depth.saturating_sub(covered)
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

/// The current video maintains a reserve of playable time ahead of
/// the playhead; time already buffered or on the wire counts toward
/// it. Without that subtraction a comfortably buffered current video
/// keeps consuming budget that upcoming candidates need.
fn current_append<'a>(
    snapshot: &PlayabilitySnapshot,
    candidate: &'a CandidateSnapshot,
    emergency: bool,
    storage_room: u64,
) -> AppendInputs<'a> {
    AppendInputs {
        candidate,
        target_ms: current_target_ms(snapshot, candidate, emergency),
        emergency,
        budget: storage_room,
    }
}

/// Current work outranks speculation but never the store's hard limit:
/// it may write into free space plus whatever this plan's evictions
/// release, not beyond it.
fn current_storage_room(snapshot: &PlayabilitySnapshot, plan: &AllocationPlan) -> u64 {
    let released: u64 = plan
        .evictions
        .iter()
        .map(|eviction| eviction.range.len())
        .sum();
    snapshot.storage.available_bytes().saturating_add(released)
}

fn inflight_playable_ms(candidate: &CandidateSnapshot) -> u64 {
    candidate
        .in_flight
        .iter()
        .filter(|active| active.identity_current)
        .map(|active| playable_gain(candidate, active.bytes))
        .sum()
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
