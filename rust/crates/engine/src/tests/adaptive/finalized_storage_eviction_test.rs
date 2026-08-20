use crate::adaptive::{
    AdaptivePlayabilityPolicy, CandidateSnapshot, PlayableRange, StorageSnapshot,
};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn finalized_candidates_overshoot_pressure_as_one_unscaled_object() {
    let policy = AdaptivePlayabilityPolicy;
    let mut sparse = snapshot(2, 20_000_000, 20_000, 2);
    sparse.storage = StorageSnapshot::new(2_000_000, 1_990_000);
    stored_candidate(&mut sparse.candidates[1], 250_000, 2_000, false);
    let mut finalized = sparse.clone();
    finalized.candidates[1].finalized = true;

    let finalized_plan = policy.plan(&finalized);
    let sparse_plan = policy.plan(&sparse);

    assert_eq!(finalized_plan.evictions.len(), 1);
    assert_eq!(
        finalized_plan.evictions[0].range,
        ByteRange::new(0, 250_000)
    );
    assert_eq!(
        finalized_plan.evictions[0].expected_playable_loss_ms,
        1_760.0
    );
    assert_eq!(
        sparse_plan.evictions[0].range,
        ByteRange::new(240_000, 250_000)
    );
}

#[test]
fn lower_density_whole_wins_even_when_its_total_loss_is_higher() {
    let policy = AdaptivePlayabilityPolicy;
    let mut evidence = snapshot(3, 20_000_000, 20_000, 2);
    evidence.storage = StorageSnapshot::new(109, 110);
    stored_candidate(&mut evidence.candidates[1], 100, 100, true);
    stored_candidate(&mut evidence.candidates[2], 10, 100, false);

    let plan = policy.plan(&evidence);

    assert_eq!(plan.evictions.len(), 1);
    assert_eq!(plan.evictions[0].range, ByteRange::new(0, 100));
    assert_eq!(plan.evictions[0].post.as_str(), "p1");
}

fn stored_candidate(candidate: &mut CandidateSnapshot, bytes: u64, ms: u64, finalized: bool) {
    let range = ByteRange::new(0, bytes);
    candidate.startup = None;
    candidate.total_bytes = Some(bytes);
    candidate.duration_ms = ms;
    candidate.playable_ranges = vec![PlayableRange {
        bytes: range,
        playable_ms: ms,
    }];
    candidate.present = vec![range];
    candidate.finalized = finalized;
}
