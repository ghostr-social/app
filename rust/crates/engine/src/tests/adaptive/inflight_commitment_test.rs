use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn a_useful_live_commitment_survives_small_network_replanning_without_duplication() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(8, 18_000_000, 20_000, 2);
    input.candidates[1].in_flight.push(InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 250_000),
        "https://origin.example/media",
        12_000,
        true,
    ));

    let plan = policy.plan(&input);

    assert_eq!(plan.retained.len(), 1, "{plan:#?}");
    assert_eq!(
        plan.retained[0].request.requested_bytes(),
        ByteRange::new(0, 250_000)
    );
    assert!(plan.allocations.iter().all(|work| {
        work.request.requested_bytes() != plan.retained[0].request.requested_bytes()
            || work.post != plan.retained[0].post
    }));
}
