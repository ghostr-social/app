use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction};
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

#[test]
fn next_item_transfer_survives_an_expired_estimated_commitment() {
    let mut input = snapshot(3, 2_500_000, 20_000, 120);
    input.observed_at_ms = 30_000;
    let next = &mut input.candidates[1];
    let action = ActionId::new(91);
    next.in_flight.push(InFlightAction::range(
        action,
        ByteRange::new(0, 65_536),
        &next.origins[0].source,
        5_000,
        true,
    ));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan.retained.iter().any(|work| work.action_id == action));
    assert!(plan.allocations.iter().all(|work| {
        work.post != input.candidates[1].post || work.request.requested_bytes().start >= 65_536
    }));
}
