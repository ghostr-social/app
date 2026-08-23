use crate::adaptive::{candidate_snapshot_at, CandidateEvidence, FeedOffset, ViewProbability};
use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::tests::adaptive_support::healthy_origin;
use crate::tests::support::progressive_meta;
use crate::{EngineParams, PostId};

const SOURCE: &str = "https://host.example/video.mp4";
const OBSERVED_AT_MS: u64 = 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn candidate_bitrate_does_not_reuse_a_stale_raw_size() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(None, Some(8_000)));
    let identity = binding.transfer(SOURCE).expect("source identity");
    let response = HttpObservation::new(
        LearnedFacts {
            content_length: Some(1_000_000),
            accept_ranges: Some(true),
            host: None,
        },
        None,
        OBSERVED_AT_MS,
        None,
    );
    assert!(catalog.learn_action_response_observation_for(&identity, response));
    let params = EngineParams::default();
    let candidate =
        candidate_snapshot_at(&catalog, &params, evidence(post), OBSERVED_AT_MS + DAY_MS)
            .expect("candidate");

    assert_eq!(candidate.total_bytes, None);
    assert_eq!(candidate.bitrate_bps, params.assumed_bitrate_bps);
}

fn evidence(post: PostId) -> CandidateEvidence {
    CandidateEvidence {
        post,
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).unwrap(),
        present: Vec::new(),
        stored_total: None,
        continuation_source: None,
        independent_object_sources: Default::default(),
        recently_evicted: Vec::new(),
        in_flight: Vec::new(),
        origins: vec![healthy_origin(SOURCE, 20_000_000, 50)],
    }
}
