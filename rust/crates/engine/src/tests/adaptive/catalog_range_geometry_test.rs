use crate::adaptive::{
    candidate_snapshot, CandidateEvidence, FeedOffset, MediaLayout, ViewProbability,
};
use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::support::progressive_meta;
use crate::{EngineParams, PostId};

#[test]
fn only_observed_range_capability_exposes_chunk_opportunities() {
    let post = PostId::new("streamable");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(
        post.clone(),
        progressive_meta(Some(3_000_000), Some(12_000)),
    );
    let source = "https://host.example/video.mp4";
    let identity = binding.transfer(source).expect("bound source");
    catalog.learn_head_for(
        &identity,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    let advisory = candidate_snapshot(&catalog, &EngineParams::default(), evidence(post.clone()))
        .expect("candidate");
    assert_eq!(advisory.layout, MediaLayout::Unknown);
    catalog.learn_response_for(
        &identity,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    let params = EngineParams {
        chunk_bytes: 1_000_000,
        ..EngineParams::default()
    };

    let candidate =
        candidate_snapshot(&catalog, &params, evidence(post)).expect("valid test fixture");

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

#[test]
fn advertised_range_denial_does_not_force_complete_layout() {
    let post = PostId::new("advisory-denial");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(
        post.clone(),
        progressive_meta(Some(3_000_000), Some(12_000)),
    );
    let identity = binding
        .transfer("https://host.example/video.mp4")
        .expect("bound source");
    catalog.learn_head_for(
        &identity,
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
    );

    let candidate = candidate_snapshot(&catalog, &EngineParams::default(), evidence(post))
        .expect("valid test fixture");

    assert_eq!(candidate.layout, MediaLayout::Unknown);
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
