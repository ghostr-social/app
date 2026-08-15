use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn streamable_allocation_uses_the_exact_storage_bytes_left_after_current_safety() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(400_000, 0);

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let mut upcoming: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == PostId::new("p1"))
        .collect();
    upcoming.sort_by_key(|work| work.range.start);

    let total: u64 = upcoming.iter().map(|work| work.range.len()).sum();
    assert_eq!(total, 396_000, "{plan:#?}");
    assert_eq!(upcoming.first().expect("upcoming work").range.start, 0);
    assert!(
        upcoming
            .windows(2)
            .all(|pair| pair[0].range.end == pair[1].range.start),
        "{plan:#?}"
    );
    assert!(upcoming
        .iter()
        .all(|work| work.expected_playable_gain_ms >= 1));
}
