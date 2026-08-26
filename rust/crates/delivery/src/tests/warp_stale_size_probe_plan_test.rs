use crate::tests::warp_head_probe_context_fixture::{generates_head, plan_at, state};
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::PostId;
use std::collections::HashSet;

const SOURCE: &str = "https://media.example/video.mp4";
const OBSERVED_AT_MS: u64 = 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn completed_head_history_rearms_after_its_size_evidence_stales() {
    let post = PostId::new("post");
    let mut state = state(post.clone(), SOURCE);
    let identity = state.catalog().transfer_identity(&post, SOURCE).expect("valid test fixture");
    let head = HttpObservation::new(
        LearnedFacts {
            content_length: Some(16),
            ..LearnedFacts::default()
        },
        None,
        OBSERVED_AT_MS,
        None,
    );
    assert!(state.catalog_mut().learn_head_observation_for(&identity, head));
    let completed = HashSet::from([identity]);

    let work = plan_at(&state, &[], &completed, OBSERVED_AT_MS + DAY_MS, 2);

    assert!(generates_head(work));
}
