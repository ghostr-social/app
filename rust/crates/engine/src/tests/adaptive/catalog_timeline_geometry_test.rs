use crate::adaptive::{candidate_snapshot, CandidateEvidence, FeedOffset, ViewProbability};
use crate::catalog::{Catalog, LearnedFacts};
use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_support::classic_moov;
use crate::tests::support::progressive_meta;
use crate::{ByteRange, EngineParams, PostId};

#[test]
fn parsed_timing_exposes_exact_sparse_media_ranges_to_the_policy() {
    let post = PostId::new("timed");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(Some(20_000), Some(2_000)));
    catalog.learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    let moov = classic_moov(&[100, 500], &[100, 100]);
    let timeline =
        parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).expect("valid test fixture");
    assert!(catalog.learn_timeline_for(&binding, timeline));

    let candidate = candidate_snapshot(&catalog, &EngineParams::default(), evidence(post))
        .expect("valid test fixture");
    let ranges: Vec<_> = candidate
        .playable_ranges
        .iter()
        .map(|item| item.bytes)
        .collect();

    assert!(ranges.contains(&ByteRange::new(100, 200)), "{ranges:?}");
    assert!(ranges.contains(&ByteRange::new(500, 600)), "{ranges:?}");
    assert!(!ranges
        .iter()
        .any(|range| range.start < 500 && range.end > 200));
}

fn evidence(post: PostId) -> CandidateEvidence {
    CandidateEvidence {
        post,
        feed_offset: FeedOffset::new(1),
        view_probability: ViewProbability::new(0.8).expect("valid test fixture"),
        present: Vec::new(),
        stored_total: None,
        continuation_source: None,
        independent_object_sources: Default::default(),
        recently_evicted: Vec::new(),
        in_flight: Vec::new(),
        origins: Vec::new(),
    }
}
