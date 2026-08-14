use crate::adaptive::{
    candidate_snapshot, CandidateEvidence, FeedOffset, MediaLayout, ViewProbability,
};
use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::support::progressive_meta;
use crate::{EngineParams, PostId};

#[test]
fn range_capable_media_exposes_duration_weighted_chunk_opportunities() {
    let post = PostId::new("streamable");
    let mut catalog = Catalog::new();
    catalog.upsert(
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
    let params = EngineParams {
        chunk_bytes: 1_000_000,
        ..EngineParams::default()
    };

    let candidate = candidate_snapshot(&catalog, &params, evidence(post)).unwrap();

    assert_eq!(candidate.layout, MediaLayout::Streamable);
    assert_eq!(candidate.playable_ranges.len(), 3);
    assert_eq!(
        candidate
            .playable_ranges
            .iter()
            .map(|range| range.playable_ms)
            .sum::<u64>(),
        12_000
    );
    assert_eq!(
        candidate
            .playable_ranges
            .iter()
            .map(|range| range.bytes.len())
            .sum::<u64>(),
        3_000_000
    );
}

fn evidence(post: PostId) -> CandidateEvidence {
    CandidateEvidence {
        post,
        feed_offset: FeedOffset::new(1),
        view_probability: ViewProbability::new(0.8).unwrap(),
        present: Vec::new(),
        recently_evicted: Vec::new(),
        in_flight: Vec::new(),
        origins: Vec::new(),
    }
}
