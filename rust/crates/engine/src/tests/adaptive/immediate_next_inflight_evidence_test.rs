use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightRange, NextReserveEvidence};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn immediate_next_covered_only_by_a_transfer_is_not_reported_ready() {
    let mut input = snapshot(2, 700_000, 0, 2);
    input.candidates[1].in_flight = vec![InFlightRange {
        bytes: ByteRange::new(0, 3_750_000),
        source: "origin".into(),
        committed_until_ms: 20_000,
        identity_current: true,
    }];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(
        plan.next_reserve,
        NextReserveEvidence::InFlight {
            post: PostId::new("p1"),
        }
    );
}
