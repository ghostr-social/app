use crate::adaptive::{AdaptivePlayabilityPolicy, AllocationReason};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn bounded_tail_layout_probe_precedes_ordinary_upcoming_media() {
    let mut snapshot = snapshot(2, 20_000_000, 20_000, 2);
    let candidate = &mut snapshot.candidates[1];
    candidate.timeline_probe = candidate.playable_ranges.last().copied();
    let tail = ByteRange::new(3_500_000, 3_750_000);

    let plan = AdaptivePlayabilityPolicy.plan(&snapshot);
    let upcoming: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == PostId::new("p1"))
        .collect();

    assert_eq!(upcoming[0].range, tail);
    assert_eq!(upcoming[0].reason, AllocationReason::MediaLayoutDiscovery);
    assert_eq!(upcoming.iter().filter(|work| work.range == tail).count(), 1);
}
