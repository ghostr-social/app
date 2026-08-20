use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction, NextReserveEvidence};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn immediate_next_covered_only_by_a_transfer_is_not_reported_ready() {
    let mut input = snapshot(2, 700_000, 0, 2);
    input.candidates[1].in_flight = vec![InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 3_750_000),
        "origin",
        20_000,
        true,
    )];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(
        plan.next_reserve,
        NextReserveEvidence::InFlight {
            post: PostId::new("p1"),
        }
    );
}
