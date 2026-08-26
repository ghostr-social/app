use super::reserve_support::{planned, record};
use crate::adaptive::{
    DecisionRecord, DecisionReplayStatus, RecordedRescueTimingQuantile, RecordedWarpReserve,
    ReserveConstraint,
};

#[test]
fn protected_rescue_records_exact_private_capacity_and_chance_evidence() {
    let (state, decision) = planned();
    let record = record(&state, &decision);
    let reserve = &record
        .warp_decision
        .as_ref()
        .expect("valid test fixture")
        .reserve;
    let source = &decision.reserve;
    let chance = reserve
        .chance
        .as_ref()
        .unwrap_or_else(|| panic!("chance-feasible rescue: {source:#?}"));

    assert_capacity(reserve, source);
    assert_authorities(reserve);
    assert_chance(chance, state.commitment_ms);
    assert_replay(&record);
}

fn assert_capacity(reserve: &RecordedWarpReserve, source: &ReserveConstraint) {
    assert!(!reserve.degraded);
    assert_eq!(
        reserve.reserved_request_slots,
        source.reserved_request_slots
    );
    assert_eq!(
        reserve.reserved_network_bytes,
        source.reserved_network_bytes
    );
    assert_eq!(
        reserve.reserved_storage_bytes,
        source.reserved_storage_bytes
    );
    assert_eq!(reserve.reserved_cpu_ms, 17);
    assert_eq!(reserve.global_request_width, 3);
    assert_eq!(reserve.protected_action_ids, source.protected_action_ids);
}

fn assert_authorities(reserve: &RecordedWarpReserve) {
    assert_eq!(reserve.authority_occupancy.len(), 2);
    assert!(reserve
        .authority_occupancy
        .iter()
        .any(|item| { item.occupied_request_slots == 1 && item.request_width == 2 }));
    assert!(reserve
        .authority_occupancy
        .iter()
        .any(|item| { item.occupied_request_slots == 0 && item.request_width == 2 }));
    assert!(reserve
        .authority_occupancy
        .iter()
        .all(|item| item.authority_id.contains(".invalid")));
}

fn assert_chance(chance: &crate::adaptive::RecordedRescueChanceEvidence, deadline_ms: u64) {
    assert_eq!(chance.deadline_ms, deadline_ms);
    assert_eq!(chance.threshold_bps, 9_500);
    assert_eq!(chance.achieved_success_bps, 9_759);
    assert_eq!(chance.transport_success_bps, 9_959);
    assert!(chance.achieved_success_bps >= chance.threshold_bps);
    assert!(chance.timing_completion_ms <= chance.deadline_ms);
    assert!(matches!(
        chance.timing_quantile,
        RecordedRescueTimingQuantile::P99
    ));
}

fn assert_replay(record: &DecisionRecord) {
    assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);
    assert!(record.replay_warp_search().is_ok());
    let json = serde_json::to_string(&record).expect("valid test fixture");
    assert!(!json.contains("origin.example") && !json.contains("active.example"));
}
