use crate::adaptive::{candidate_snapshot, CandidateEvidence, FeedOffset, ViewProbability};
use crate::catalog::{Catalog, LearnedFacts};
use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::adaptive_support::healthy_origin;
use crate::tests::media_timeline_support::classic_mdat_prefix;
use crate::tests::support::progressive_meta;
use crate::{ByteRange, EngineParams, PostId};

pub(super) fn candidate(
    ftyp: &[u8],
    moov: &[u8],
    present: Vec<ByteRange>,
) -> crate::adaptive::CandidateSnapshot {
    let post = PostId::new("p1");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(Some(20_000), Some(2_000)));
    catalog.learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..Default::default()
        },
    );
    let prefix = classic_mdat_prefix(ftyp, 10_000, 24);
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(10_000, moov),
    ])
    .expect("valid test fixture");
    assert!(catalog.learn_timeline_for(&binding, timeline));
    candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(1),
            view_probability: ViewProbability::new(0.8).expect("valid test fixture"),
            present,
            stored_total: Some(20_000),
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![healthy_origin(
                "https://host.example/video.mp4",
                20_000_000,
                20,
            )],
        },
    )
    .expect("valid test fixture")
}

pub(super) fn metadata(moov: &[u8]) -> Vec<ByteRange> {
    vec![
        ByteRange::new(0, 24),
        ByteRange::new(10_000, 10_000 + moov.len() as u64),
    ]
}

pub(super) fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
