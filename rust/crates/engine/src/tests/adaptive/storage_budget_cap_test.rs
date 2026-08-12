use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;

#[test]
fn available_storage_caps_total_safe_plan_bytes() {
    let mut input = snapshot(8, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(1_000_000, 0);

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let allocated: u64 = plan
        .allocations
        .iter()
        .map(|allocation| allocation.range.len())
        .sum();

    assert!(allocated <= input.storage.available_bytes(), "{allocated}");
}
