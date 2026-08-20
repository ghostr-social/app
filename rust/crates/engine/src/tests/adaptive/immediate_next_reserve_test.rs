use crate::adaptive::{
    AdaptivePlayabilityPolicy, AllocationReason, NetworkSnapshot, NextReserveEvidence,
};
use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn constrained_startup_reserves_useful_immediate_next_work_before_deep_current() {
    let mut input = snapshot(2, 700_000, 0, 2);
    input.playback.phase = PlaybackPhase::Starting;
    input.network = NetworkSnapshot {
        throughput_bps: 700_000,
        rtt_ms: 450,
        packet_loss_bps: 0,
        connection_capacity: 1,
        connection_ceiling: 1,
        confidence: EstimateConfidence::High,
    };

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let current = PostId::new("p0");
    let next = PostId::new("p1");
    let next_index = plan
        .allocations
        .iter()
        .position(|work| work.post == next)
        .expect("immediate-next reserve");

    assert_eq!(plan.allocations[0].post, current);
    assert!(!plan.allocations[next_index]
        .request
        .requested_bytes()
        .is_empty());
    assert!(plan.allocations[next_index].expected_playable_gain_ms > 0);
    assert_eq!(
        plan.allocations[next_index].reason,
        AllocationReason::NextStartability,
    );
    assert!(plan
        .allocations
        .iter()
        .skip(next_index + 1)
        .any(|work| work.post == current));
    assert!(matches!(
        plan.next_reserve,
        NextReserveEvidence::Granted { post, .. } if post == next
    ));
}
