use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn current_useful_inflight_work_survives_an_emergency_replan() {
    let mut input = snapshot(3, 20_000_000, 1_000, 2);
    input.candidates[0].in_flight.push(InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 250_000),
        "origin",
        12_000,
        true,
    ));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .retained
        .iter()
        .any(|work| work.post == PostId::new("p0")));
}

#[test]
fn useful_current_work_survives_after_its_minimum_commitment() {
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    input.candidates[0].in_flight.push(InFlightAction::range(
        crate::ActionId::new(2),
        ByteRange::new(0, 250_000),
        "origin",
        input.observed_at_ms,
        true,
    ));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .retained
        .iter()
        .any(|work| work.post == PostId::new("p0")));
}
