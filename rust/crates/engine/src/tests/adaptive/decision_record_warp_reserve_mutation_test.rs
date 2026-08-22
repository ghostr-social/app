use super::reserve_support::{planned, record};
use crate::adaptive::{
    DecisionReplayStatus, RescueTimingQuantile, ReserveConstraint, ReserveDegradedReason,
    WarpPlanningDecision,
};

#[test]
fn protected_rescue_capacity_mutations_fail_authenticated_coherence() {
    let (state, decision) = planned();

    rejects(&state, &decision, |r| r.reserved_request_slots ^= 1);
    rejects(&state, &decision, |r| r.reserved_network_bytes += 1);
    rejects(&state, &decision, |r| r.reserved_storage_bytes += 1);
    rejects(&state, &decision, |r| r.reserved_cpu_ms += 1);
    rejects(&state, &decision, |r| r.global_request_width += 1);
    rejects(&state, &decision, |r| {
        r.authority_occupancy[0].occupied_request_slots += 1;
    });
    rejects(&state, &decision, |r| {
        r.authority_occupancy[0].request_width += 1;
    });
    rejects(&state, &decision, |r| r.protected_action_ids.push(999));
    rejects(&state, &decision, |r| {
        r.authority_occupancy.push(r.authority_occupancy[0].clone())
    });
    rejects(&state, &decision, |r| {
        r.authority_occupancy
            .retain(|item| item.occupied_request_slots > 0)
    });
}

#[test]
fn protected_rescue_chance_mutations_fail_authenticated_coherence() {
    let (state, decision) = planned();

    rejects(&state, &decision, |r| chance(r).deadline_ms += 1);
    rejects(&state, &decision, |r| chance(r).threshold_bps ^= 1);
    rejects(&state, &decision, |r| {
        chance(r).achieved_success_bps ^= 1;
    });
    rejects(&state, &decision, |r| {
        chance(r).transport_success_bps ^= 1;
    });
    rejects(&state, &decision, |r| {
        chance(r).timing_completion_ms += 1;
    });
    rejects(&state, &decision, |r| {
        chance(r).timing_quantile = RescueTimingQuantile::P95;
    });
    rejects(&state, &decision, |r| {
        r.degraded = true;
        r.degraded_reason = Some(ReserveDegradedReason::NoFeasibleRescue);
    });
}

fn chance(reserve: &mut ReserveConstraint) -> &mut crate::adaptive::RescueChanceEvidence {
    reserve.chance.as_mut().unwrap()
}

fn rejects(
    state: &crate::adaptive::PlayabilitySnapshot,
    source: &WarpPlanningDecision,
    mutate: impl Fn(&mut ReserveConstraint),
) {
    let mut decision = source.clone();
    mutate(&mut decision.reserve);
    mutate(&mut decision.search_replay.as_mut().unwrap().reserve);
    let captured = record(state, &decision);
    assert_eq!(captured.replay(), DecisionReplayStatus::PlanMismatch);
    assert_eq!(
        captured.replay_warp_search(),
        Err(DecisionReplayStatus::PlanMismatch)
    );
}
