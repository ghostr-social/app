use crate::probe::pool::MetadataProbePool;
use crate::tests::warp_head_probe_context_fixture::{
    generates_head, plan_at, state_with_size,
};
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::PostId;

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn planner_reprobes_after_redirect_changes_with_the_same_validator() {
    let post = PostId::new("post");
    let mut state = state_with_size(post.clone(), SOURCE, 16);
    let identity = state.catalog().transfer_identity(&post, SOURCE).expect("valid test fixture");
    assert!(state.catalog_mut().learn_head_observation_for(
        &identity,
        observation(Some(true), "https://cdn-a.example/video.mp4", 1)
    ));
    let mut probes = MetadataProbePool::new(1);
    probes.learned(&identity, state.catalog().http_generation_for(&identity));
    assert!(state.catalog_mut().learn_response_observation_for(
        &identity,
        observation(None, "https://cdn-b.example/video.mp4", 2)
    ));
    let completed = probes.current_completed_identities(state.catalog());

    assert!(generates_head(plan_at(&state, &[], &completed, 2, 2)));
}

fn observation(ranges: Option<bool>, final_url: &str, at: u64) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: ranges.map(|_| 16),
            accept_ranges: ranges,
            host: None,
        },
        Some("video/mp4".into()),
        at,
        EvidenceValidator::strong_etag("\"v1\""),
    )
    .with_final_url(final_url)
}
