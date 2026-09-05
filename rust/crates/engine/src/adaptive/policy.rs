use super::allocation::{append_candidate, AppendInputs};
use super::commitments::retained;
use super::current_lane::{current_need_bytes, CurrentLane};
use super::eviction::evictions;
use super::frontier::{discovery_demand, upcoming_candidates};
use super::plan::{AllocationPlan, DiscoveryDemand, NextReserveEvidence};
use super::ranges::missing;
use super::reserve_window::{build as build_ready_reserve, ReserveInputs, ReserveModePolicy};
use super::reserves::{critical_slots, planned_bytes, planned_gain, sibling_planned_bytes};
use super::resources::{endangered, speculative_budget, upcoming_depth_ms};
use super::{CandidateSnapshot, PlayabilitySnapshot};

const EMERGENCY_TRANSITION_DEPTH_MS: u64 = 2_000;

#[derive(Clone, Copy, Debug, Default)]
pub struct AdaptivePlayabilityPolicy;

impl AdaptivePlayabilityPolicy {
    pub fn plan(self, snapshot: &PlayabilitySnapshot) -> AllocationPlan {
        Self::plan_with_reserve(snapshot, ReserveModePolicy::Ordered)
    }

    fn plan_with_reserve(
        snapshot: &PlayabilitySnapshot,
        reserve_policy: ReserveModePolicy,
    ) -> AllocationPlan {
        let Some(current) = current_candidate(snapshot) else {
            return hls_frontier_plan(snapshot, reserve_policy);
        };
        let emergency = snapshot.playback.authority == super::CurrentAuthority::Canonical
            && endangered(snapshot, current);
        let mut plan = initial_plan(snapshot, current_need_bytes(snapshot, current, emergency));
        let lane = CurrentLane::new(snapshot, current, emergency, &plan);
        lane.append_start(&mut plan, snapshot);
        reserve_next(&mut plan, snapshot, &lane, reserve_policy);
        lane.append_depth(&mut plan, snapshot);
        append_followup(&mut plan, snapshot, &lane);
        plan.retained = retained(snapshot, emergency, critical_slots(&plan));
        finalize(&mut plan, snapshot);
        plan
    }
}

fn reserve_next(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    lane: &CurrentLane<'_>,
    mode_policy: ReserveModePolicy,
) {
    let protected = !lane.emergency || lane.protected(plan);
    build_ready_reserve(
        plan,
        snapshot,
        ReserveInputs {
            transfer_budget: speculative_budget(snapshot),
            storage_room: lane.storage_room,
            current_emergency: lane.emergency,
            current_protected: protected,
            mode_policy,
        },
    );
}

fn append_followup(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    lane: &CurrentLane<'_>,
) {
    if !lane.emergency {
        return append_upcoming(plan, snapshot, lane.storage_room);
    }
    let next_ready = matches!(
        plan.next_reserve,
        NextReserveEvidence::Ready { .. } | NextReserveEvidence::Structural { .. }
    );
    if lane.protected(plan) && next_ready {
        append_emergency_transition(plan, snapshot, lane.storage_room);
    }
}

fn append_upcoming(plan: &mut AllocationPlan, snapshot: &PlayabilitySnapshot, storage_room: u64) {
    for candidate in upcoming_candidates(snapshot)
        .into_iter()
        .filter(|candidate| super::reserve_model::allows_progressive_followup(snapshot, candidate))
    {
        let transfer =
            speculative_budget(snapshot).saturating_sub(sibling_planned_bytes(plan, snapshot));
        let storage = storage_room.saturating_sub(planned_bytes(plan));
        let budget = transfer.min(storage);
        if budget == 0 {
            return;
        }
        let target = upcoming_depth_ms(snapshot).saturating_sub(planned_gain(plan, candidate));
        append_candidate(
            plan,
            snapshot,
            AppendInputs {
                candidate,
                target_ms: target,
                emergency: false,
                budget,
                reason: None,
            },
        );
    }
}

fn append_emergency_transition(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    storage_room: u64,
) {
    let transfer =
        speculative_budget(snapshot).saturating_sub(sibling_planned_bytes(plan, snapshot));
    let budget = transfer.min(storage_room.saturating_sub(planned_bytes(plan)));
    let candidate = upcoming_candidates(snapshot)
        .into_iter()
        .filter(|candidate| super::reserve_model::allows_progressive_followup(snapshot, candidate))
        .find(|candidate| !missing(candidate).is_empty());
    if let Some(candidate) = candidate {
        append_candidate(
            plan,
            snapshot,
            AppendInputs {
                candidate,
                target_ms: EMERGENCY_TRANSITION_DEPTH_MS,
                emergency: false,
                budget,
                reason: None,
            },
        );
    }
}

fn current_candidate(snapshot: &PlayabilitySnapshot) -> Option<&CandidateSnapshot> {
    snapshot
        .candidates
        .iter()
        .find(|candidate| candidate.post == snapshot.playback.current)
}

fn initial_plan(snapshot: &PlayabilitySnapshot, current_need_bytes: u64) -> AllocationPlan {
    AllocationPlan {
        evictions: evictions(snapshot, current_need_bytes),
        ..AllocationPlan::default()
    }
}

fn empty_frontier_plan() -> AllocationPlan {
    AllocationPlan {
        discovery_demand: DiscoveryDemand::Expand,
        ..AllocationPlan::default()
    }
}

fn hls_frontier_plan(
    snapshot: &PlayabilitySnapshot,
    mode_policy: ReserveModePolicy,
) -> AllocationPlan {
    let mut plan = empty_frontier_plan();
    let Some(current) = current_hls(snapshot) else {
        return plan;
    };
    let current_at_risk = snapshot.playback.authority == super::CurrentAuthority::Canonical
        && !current.player_ready();
    build_ready_reserve(
        &mut plan,
        snapshot,
        ReserveInputs {
            transfer_budget: speculative_budget(snapshot),
            storage_room: snapshot.storage.available_bytes(),
            current_emergency: current_at_risk,
            current_protected: !current_at_risk,
            mode_policy,
        },
    );
    finalize(&mut plan, snapshot);
    plan
}

fn current_hls(snapshot: &PlayabilitySnapshot) -> Option<&super::HlsCandidateSnapshot> {
    snapshot
        .hls_candidates
        .iter()
        .find(|candidate| candidate.post == snapshot.playback.current)
}

fn finalize(plan: &mut AllocationPlan, snapshot: &PlayabilitySnapshot) {
    let budget = speculative_budget(snapshot);
    plan.discovery_demand = discovery_demand(snapshot, plan, budget);
}
