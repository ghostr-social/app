use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightRange};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn current_useful_inflight_work_survives_an_emergency_replan() {
    let mut input = snapshot(3, 20_000_000, 1_000, 2);
    input.candidates[0].in_flight.push(InFlightRange {
        bytes: ByteRange::new(0, 250_000),
        source: "origin".to_owned(),
        committed_until_ms: 12_000,
        identity_current: true,
    });

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .retained
        .iter()
        .any(|work| work.post == PostId::new("p0")));
}

#[test]
fn useful_current_work_survives_after_its_minimum_commitment() {
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    input.candidates[0].in_flight.push(InFlightRange {
        bytes: ByteRange::new(0, 250_000),
        source: "origin".to_owned(),
        committed_until_ms: input.observed_at_ms,
        identity_current: true,
    });

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .retained
        .iter()
        .any(|work| work.post == PostId::new("p0")));
}
