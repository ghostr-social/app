use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction, PlayabilitySnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn packet_loss_updates_cost_evidence_for_the_same_retained_range() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    add_commitment(&mut input);
    let healthy = policy.plan(&input).retained.remove(0);

    input.network.packet_loss_bps = 6_000;
    let lossy = policy.plan(&input).retained.remove(0);

    assert_eq!(lossy.post, healthy.post);
    assert_eq!(lossy.request, healthy.request);
    assert_eq!(lossy.source, healthy.source);
    assert!(
        lossy.utility.expected_delivery_ms > healthy.utility.expected_delivery_ms,
        "healthy={healthy:#?} lossy={lossy:#?}"
    );
}

fn add_commitment(input: &mut PlayabilitySnapshot) {
    input.candidates[0].in_flight.push(InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 250_000),
        "https://origin.example/media",
        12_000,
        true,
    ));
}
