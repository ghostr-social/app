use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction, StorageSnapshot};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn present_startup_bytes_survive_while_the_rest_of_the_closure_is_in_flight() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(2_000_000, 1_990_000);
    let present = ByteRange::new(0, 10_000);
    let arriving = ByteRange::new(100_000, 110_000);
    input.candidates[1].startup = StartupFootprint::new(
        vec![present, arriving],
        2_000,
        StartupProvenance::ClassicMp4V1,
    );
    input.candidates[1].present = vec![present];
    input.candidates[1].in_flight = vec![InFlightAction::range(
        crate::ActionId::new(1),
        arriving,
        "origin",
        20_000,
        true,
    )];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .evictions
        .iter()
        .all(|item| !overlaps(item.range, present)));
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
