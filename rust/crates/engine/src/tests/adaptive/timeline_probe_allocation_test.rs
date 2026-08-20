use crate::adaptive::{
    AdaptivePlayabilityPolicy, AllocationReason, PlayableRange, RetrievalRequest,
};
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

    assert_eq!(upcoming[0].request.requested_bytes(), tail);
    assert_eq!(upcoming[0].reason, AllocationReason::MediaLayoutDiscovery);
    assert_eq!(
        upcoming
            .iter()
            .filter(|work| work.request.requested_bytes() == tail)
            .count(),
        1
    );
}

#[test]
fn promotable_timeline_probe_excludes_sibling_requests_for_the_post() {
    let mut input = snapshot(2, 40_000_000, 30_000, 2);
    let candidate = &mut input.candidates[1];
    candidate.total_bytes = Some(800_000);
    candidate.playable_ranges = (0..4)
        .map(|index| PlayableRange {
            bytes: ByteRange::new(index * 200_000, (index + 1) * 200_000),
            playable_ms: 2_000,
        })
        .collect();
    candidate.timeline_probe = candidate.playable_ranges.last().copied();
    let post = candidate.post.clone();

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let work: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == post)
        .collect();

    assert_eq!(work.len(), 1, "{plan:#?}");
    assert!(matches!(
        work[0].request,
        RetrievalRequest::FetchRange {
            promotion: Some(_),
            ..
        }
    ));
}
