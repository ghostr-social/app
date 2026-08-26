use crate::adaptive::{
    candidate_snapshot, CandidateEvidence, FeedOffset, MediaLayout, ViewProbability,
};
use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, EngineParams, PostId};

#[test]
fn range_blind_probe_evidence_requires_one_complete_file_opportunity() {
    let post = PostId::new("complete");
    let mut catalog = Catalog::new();
    catalog.upsert(
        post.clone(),
        progressive_meta(Some(10_000_000), Some(60_000)),
    );
    catalog.learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
    );

    let candidate = candidate_snapshot(&catalog, &EngineParams::default(), evidence(post))
        .expect("valid test fixture");

    assert_eq!(candidate.layout, MediaLayout::RequiresCompleteFile);
    assert_eq!(candidate.playable_ranges.len(), 1);
    assert_eq!(
        candidate.playable_ranges[0].bytes,
        ByteRange::new(0, 10_000_000)
    );
    assert_eq!(candidate.playable_ranges[0].playable_ms, 60_000);
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
