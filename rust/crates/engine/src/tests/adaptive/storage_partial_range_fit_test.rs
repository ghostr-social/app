use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn streamable_allocation_uses_the_exact_storage_bytes_left_after_current_safety() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.storage = StorageSnapshot::new(400_000, 0);

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let upcoming = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("upcoming partial allocation");

    assert_eq!(upcoming.range, ByteRange::new(0, 146_000));
    assert_eq!(upcoming.expected_playable_gain_ms, 1_168);
}
