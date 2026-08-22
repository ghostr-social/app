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

    assert_eq!(first.request.requested_bytes().start, 100_000);
    assert!(first.request.requested_bytes().len() <= crate::adaptive::REQUEST_SLICE_BYTES);
    assert!(first.expected_playable_gain_ms >= 1_200, "{plan:#?}");
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

    assert_eq!(suffix.request.requested_bytes().start, 249_999);
    assert!(suffix.expected_playable_gain_ms >= 1);
}
