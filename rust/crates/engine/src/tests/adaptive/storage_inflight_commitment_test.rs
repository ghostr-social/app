use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn a_useful_transfer_that_reserves_the_last_storage_bytes_stays_committed() {
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(500_000, 245_000);
    input.candidates[0].present = vec![ByteRange::new(0, 250_000)];
    input.candidates[1].in_flight.push(InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 250_000),
        "origin",
        12_000,
        true,
    ));

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        plan.retained
            .iter()
            .any(|work| work.post == PostId::new("p1")),
        "{plan:#?}"
    );
    assert!(
        plan.allocations
            .iter()
            .all(|work| work.post == PostId::new("p0")),
        "{plan:#?}"
    );
}
