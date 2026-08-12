use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn zero_storage_capacity_never_panics_or_admits_speculation() {
    let mut input = snapshot(4, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(0, 0);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .allocations
        .iter()
        .all(|allocation| allocation.post == PostId::new("p0")));
}
