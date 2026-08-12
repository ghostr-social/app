use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::tests::adaptive_support::{frontier, snapshot};

#[test]
fn high_storage_pressure_stops_new_speculation_without_blocking_current_safety() {
    let policy = AdaptivePlayabilityPolicy;
    let roomy = policy.plan(&snapshot(12, 20_000_000, 20_000, 2));
    let mut pressured = snapshot(12, 20_000_000, 20_000, 2);
    pressured.storage = StorageSnapshot::new(2_000_000_000, 1_990_000_000);

    let pressured = policy.plan(&pressured);

    assert_eq!(frontier(&pressured).len(), 1, "{pressured:#?}");
    assert!(frontier(&roomy).len() > frontier(&pressured).len());
}
