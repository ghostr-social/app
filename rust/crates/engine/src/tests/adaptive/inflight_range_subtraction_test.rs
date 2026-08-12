use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightRange};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn new_origin_work_starts_after_a_partly_overlapping_active_transfer() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[1].in_flight.push(InFlightRange {
        bytes: ByteRange::new(0, 100_000),
        source: "origin".to_owned(),
        committed_until_ms: 12_000,
        identity_current: true,
    });

    let plan = policy.plan(&input);
    let first = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("remaining candidate work");

    assert_eq!(first.range.start, 100_000);
    assert!(first.range.len() <= crate::adaptive::REQUEST_SLICE_BYTES);
}
