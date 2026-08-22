use crate::adaptive::{candidate_snapshot, CandidateEvidence, FeedOffset, ViewProbability};
use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, EngineParams, PostId};

#[test]
fn missing_head_timing_exposes_one_bounded_tail_layout_probe() {
    let post = PostId::new("tail-timed");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(
        post.clone(),
        progressive_meta(Some(3_000_000), Some(12_000)),
    );
    catalog.learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    assert!(catalog.require_tail_timeline_for(&binding));
    let params = EngineParams {
        chunk_bytes: 1_000_000,
        ..EngineParams::default()
    };

    let candidate = candidate_snapshot(&catalog, &params, evidence(post)).unwrap();

    let probe = candidate.timeline_probe.expect("tail probe");
    assert_eq!(probe.bytes, ByteRange::new(2_000_000, 3_000_000));
    assert_eq!(probe.playable_ms, 4_000);
}

fn evidence(post: PostId) -> CandidateEvidence {
    CandidateEvidence {
        post,
        feed_offset: FeedOffset::new(1),
        view_probability: ViewProbability::new(0.8).unwrap(),
        present: Vec::new(),
        stored_total: None,
        continuation_source: None,
        independent_object_sources: Default::default(),
        recently_evicted: Vec::new(),
        in_flight: Vec::new(),
        origins: Vec::new(),
    }
}
