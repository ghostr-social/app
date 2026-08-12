use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn replanning_requests_only_the_missing_suffix_of_a_partly_stored_playable_range() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[1].present = vec![ByteRange::new(0, 100_000)];

    let plan = policy.plan(&input);
    let first = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("upcoming allocation");

    assert_eq!(first.range, ByteRange::new(100_000, 250_000));
    assert_eq!(first.expected_playable_gain_ms, 1_200);
}

#[test]
fn a_nonempty_missing_suffix_always_has_positive_playable_gain() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[1].present = vec![ByteRange::new(0, 249_999)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let suffix = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("one-byte suffix");

    assert_eq!(suffix.range, ByteRange::new(249_999, 250_000));
    assert_eq!(suffix.expected_playable_gain_ms, 1);
}
