use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn coalesced_storage_evicts_around_the_adjacent_startup_closure() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(2_000_000, 1_990_000);
    let stored = ByteRange::new(0, 1_000_000);
    let startup = ByteRange::new(100_000, 110_000);
    input.candidates[1].startup =
        StartupFootprint::new(vec![startup], 2_000, StartupProvenance::ClassicMp4V1);
    input.candidates[1].present = vec![stored];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        !plan.evictions.is_empty(),
        "unprotected bytes remain evictable"
    );
    assert!(plan
        .evictions
        .iter()
        .all(|item| !overlaps(item.range, startup)));
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
