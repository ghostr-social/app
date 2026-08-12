use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightRange};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn a_useful_live_commitment_survives_small_network_replanning_without_duplication() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(8, 18_000_000, 20_000, 2);
    input.candidates[3].in_flight.push(InFlightRange {
        bytes: ByteRange::new(0, 250_000),
        source: "origin".to_owned(),
        committed_until_ms: 12_000,
        identity_current: true,
    });

    let plan = policy.plan(&input);

    assert_eq!(plan.retained.len(), 1, "{plan:#?}");
    assert_eq!(plan.retained[0].range, ByteRange::new(0, 250_000));
    assert!(plan
        .allocations
        .iter()
        .all(|work| work.range != plan.retained[0].range || work.post != plan.retained[0].post));
}
