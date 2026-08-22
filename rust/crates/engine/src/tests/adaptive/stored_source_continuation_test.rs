use crate::adaptive::{
    candidate_snapshot, AdaptivePlayabilityPolicy, CandidateEvidence, FeedOffset, ViewProbability,
};
use crate::catalog::Catalog;
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::{ByteRange, DeliveryKind, EngineParams, PostId, VideoMeta};

const PRIMARY: &str = "https://primary/video.mp4";
const MIRROR: &str = "https://mirror/video.mp4";

#[test]
fn stored_mirror_prefix_continues_on_that_mirror_to_its_observed_extent() {
    let allocation = allocation(true);

    assert_eq!(allocation.source, MIRROR);
    assert_eq!(allocation.request.requested_bytes(), ByteRange::new(8, 16));
}

#[test]
fn unavailable_stored_source_cannot_seed_a_suffix_on_another_source() {
    let allocation = allocation(false);

    assert_eq!(allocation.source, PRIMARY);
    assert_eq!(allocation.request.requested_bytes().start, 0);
}

fn allocation(mirror_available: bool) -> crate::adaptive::Allocation {
    let candidate = candidate(mirror_available);
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.playback.current = PostId::new("post");
    input.candidates[0] = candidate;
    AdaptivePlayabilityPolicy.plan(&input).allocations.remove(0)
}

fn candidate(mirror_available: bool) -> crate::adaptive::CandidateSnapshot {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), metadata());
    let mut mirror = healthy_origin(MIRROR, 1_000_000, 50);
    mirror.available = mirror_available;
    candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(0),
            view_probability: ViewProbability::new(1.0).unwrap(),
            present: vec![ByteRange::new(0, 8)],
            stored_total: Some(16),
            continuation_source: Some(MIRROR.to_owned()),
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![healthy_origin(PRIMARY, 20_000_000, 50), mirror],
        },
    )
    .unwrap()
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![PRIMARY.to_owned(), MIRROR.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
