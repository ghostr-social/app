use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn new_origin_work_starts_after_a_partly_overlapping_active_transfer() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[1].in_flight.push(InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 100_000),
        "origin",
        12_000,
        true,
    ));

    let plan = policy.plan(&input);
    let first = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("remaining candidate work");

    assert_eq!(first.request.requested_bytes().start, 100_000);
    assert!(first.request.requested_bytes().len() <= crate::adaptive::REQUEST_SLICE_BYTES);
}
