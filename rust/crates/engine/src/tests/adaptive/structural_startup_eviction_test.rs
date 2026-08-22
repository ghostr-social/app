use crate::adaptive::{AdaptivePlayabilityPolicy, PlayerPreparation, StorageSnapshot};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn storage_pressure_preserves_the_adjacent_structural_startup_closure() {
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(2_000_000, 1_990_000);
    let media = ByteRange::new(0, 250_000);
    let initialization = ByteRange::new(1_000_000, 1_010_000);
    input.candidates[1].startup = StartupFootprint::new(
        vec![media, initialization],
        2_000,
        StartupProvenance::ClassicMp4V1,
    );
    input.candidates[1].player_preparation = PlayerPreparation::Unverified;
    input.candidates[1].present = vec![media, initialization];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .evictions
        .iter()
        .all(|item| { item.post.as_str() != "p1" || !overlaps(item.range, initialization) }));
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
