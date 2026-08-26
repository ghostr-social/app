use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot, ViewProbability};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn full_store_chooses_a_feasible_tail_instead_of_an_interior_rewrite() {
    let mut input = snapshot(3, 20_000_000, 20_000, 120);
    input.storage = StorageSnapshot::new(2_000_000, 2_000_000);
    input.candidates[1].view_probability = ViewProbability::new(0.9).expect("valid test fixture");
    input.candidates[1].present = vec![ByteRange::new(0, 350_000)];
    input.candidates[2].view_probability = ViewProbability::new(0.01).expect("valid test fixture");
    input.candidates[2].startup = StartupFootprint::new(
        vec![ByteRange::new(900_000, 1_000_000)],
        2_000,
        StartupProvenance::ClassicMp4V1,
    );
    input.candidates[2].present = vec![ByteRange::new(0, 1_000_000)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(!plan.evictions.is_empty(), "{plan:#?}");
    assert_eq!(plan.evictions[0].post, PostId::new("p1"), "{plan:#?}");
    assert_eq!(plan.evictions[0].range.end, 350_000, "{plan:#?}");
}
