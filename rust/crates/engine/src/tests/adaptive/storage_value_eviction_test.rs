use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn storage_pressure_evicts_the_lowest_value_cached_candidate_first() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(5, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(2_000_000, 1_990_000);
    input.candidates[1].present = vec![ByteRange::new(0, 250_000)];
    input.candidates[4].present = vec![ByteRange::new(0, 250_000)];

    let plan = policy.plan(&input);

    assert!(!plan.evictions.is_empty(), "{plan:#?}");
    assert_eq!(plan.evictions[0].post, PostId::new("p4"));
    assert_eq!(plan.evictions[0].range, ByteRange::new(240_000, 250_000));
}
