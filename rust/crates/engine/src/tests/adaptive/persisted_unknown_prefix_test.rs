use crate::adaptive::{
    candidate_snapshot, AdaptivePlayabilityPolicy, CandidateEvidence, FeedOffset, MediaLayout,
};
use crate::catalog::Catalog;
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::{ByteRange, DeliveryKind, EngineParams, PostId, VideoMeta};

const PREFIX: u64 = 262_144;
const TOTAL: u64 = 600_000;

#[test]
fn unknown_layout_bootstrap_starts_after_the_contiguous_prefix() {
    let candidate = candidate(vec![ByteRange::new(0, PREFIX)]);

    assert_eq!(candidate.layout, MediaLayout::Unknown);
    assert_eq!(candidate.playable_ranges[0].bytes.start, PREFIX);
    assert!(candidate.playable_ranges[0].bytes.end <= TOTAL);
}

#[test]
fn fully_stored_unknown_layout_keeps_complete_playability() {
    let candidate = candidate(vec![ByteRange::new(0, TOTAL)]);

    assert_eq!(candidate.playable_ranges[0].bytes, ByteRange::new(0, TOTAL));
}

#[test]
fn unknown_layout_plan_advances_past_the_persisted_prefix() {
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates[0] = candidate(vec![ByteRange::new(0, PREFIX)]);
    input.playback.current = PostId::new("persisted");

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.allocations[0].request.requested_bytes().start, PREFIX);
}

fn candidate(present: Vec<ByteRange>) -> crate::adaptive::CandidateSnapshot {
    let post = PostId::new("persisted");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), metadata());
    candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(0),
            view_probability: crate::adaptive::ViewProbability::new(1.0).unwrap(),
            present,
            stored_total: None,
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![healthy_origin("origin", 1_000_000, 100)],
        },
    )
    .expect("candidate")
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video.mp4".into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(TOTAL),
        duration_ms: Some(10_000),
    }
}
