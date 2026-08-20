use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn speculative_storage_budget_reserves_bytes_already_in_flight() {
    let mut input = snapshot(8, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(1_000_000, 200_000);
    input.candidates[3].in_flight.push(InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 250_000),
        "origin",
        12_000,
        true,
    ));

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let newly_planned: u64 = plan
        .allocations
        .iter()
        .map(|work| work.request.reserved_network_bytes())
        .sum();

    assert!(newly_planned <= 540_000, "{newly_planned}: {plan:#?}");
}
