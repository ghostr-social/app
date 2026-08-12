use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightRange};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn paid_future_work_uses_spare_hard_capacity_when_current_needs_no_lane() {
    let mut input = snapshot(3, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 3;
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    input.candidates[1].in_flight.push(active());
    input.candidates[2].in_flight.push(active());

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 2, "{plan:#?}");
}

#[test]
fn paid_future_work_uses_capacity_left_beside_one_current_lane() {
    let mut input = snapshot(3, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 3;
    input.candidates[0].in_flight.push(InFlightRange {
        bytes: ByteRange::new(0, 3_750_000),
        ..active()
    });
    input.candidates[1].in_flight.push(active());
    input.candidates[2].in_flight.push(active());

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 3, "{plan:#?}");
}

fn active() -> InFlightRange {
    InFlightRange {
        bytes: ByteRange::new(0, 250_000),
        source: "origin".to_owned(),
        committed_until_ms: 12_000,
        identity_current: true,
    }
}
