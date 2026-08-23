use crate::tests::warp_head_probe_context_fixture::{generates_head, plan_at, state};
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::PostId;
use std::collections::HashSet;

const SOURCE: &str = "https://media.example/video.mp4";
const OBSERVED_AT_MS: u64 = 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn stale_range_evidence_does_not_repeat_an_uninformative_head() {
    let post = PostId::new("post");
    let mut state = state(post.clone(), SOURCE);
    let identity = state.catalog().transfer_identity(&post, SOURCE).unwrap();
    let observation = HttpObservation::new(
        LearnedFacts {
            content_length: Some(16),
            accept_ranges: Some(false),
            host: None,
        },
        Some("video/mp4".to_owned()),
        OBSERVED_AT_MS,
        EvidenceValidator::strong_etag("\"generation-1\""),
    );
    assert!(state
        .catalog_mut()
        .learn_response_observation_for(&identity, observation));
    let completed = HashSet::from([identity]);

    let work = plan_at(&mut state, &[], &completed, OBSERVED_AT_MS + DAY_MS, 2);

    assert!(!generates_head(work));
}
