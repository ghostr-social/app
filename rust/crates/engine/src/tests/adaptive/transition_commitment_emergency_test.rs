use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn emergency_retains_a_transition_when_current_needs_no_connection() {
    let mut input = snapshot(2, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 1;
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    input.candidates[1].in_flight.push(active(250_000));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .retained
        .iter()
        .any(|work| work.post == PostId::new("p1")));
}

#[test]
fn emergency_retains_one_paid_transition_beside_current_work() {
    let mut input = snapshot(2, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 1;
    input.candidates[1].in_flight.push(active(250_000));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        plan.retained
            .iter()
            .any(|work| work.post == PostId::new("p1")),
        "{plan:#?}"
    );
}

#[test]
fn emergency_retains_transition_beside_paid_current_work() {
    let mut input = snapshot(2, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 1;
    input.candidates[0].in_flight.push(active(3_750_000));
    input.candidates[1].in_flight.push(active(250_000));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 2, "{plan:#?}");
}

#[test]
fn hard_single_connection_reserves_the_lane_for_current_work() {
    let mut input = snapshot(2, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 1;
    input.candidates[0].in_flight.push(active(3_750_000));
    input.candidates[1].in_flight.push(active(250_000));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 1, "{plan:#?}");
    assert_eq!(plan.retained[0].post, PostId::new("p0"));
}

fn active(end: u64) -> InFlightAction {
    InFlightAction::range(
        crate::ActionId::new(end),
        ByteRange::new(0, end),
        "origin",
        12_000,
        true,
    )
}
