
use crate::delivery_events::{DeliveryCandidate, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState};
use crate::tests::player_preparation_fixture::{meta, state};
use ghostr_engine::adaptive::PlannerCapability;
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::PostId;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_partial_store::partial_range_store::ContentRevision;

const POST: &str = "next";
const SOURCE: &str = "https://media.example/next.mp4";

#[test]
fn hashless_decoder_rejection_expires_when_validator_generation_changes() {
    let mut state = state(&[POST], 0);
    let high = VideoRendition::try_new(meta(POST), Some(6_000_000)).expect("valid test fixture");
    let low = VideoRendition::try_new(meta("low"), Some(1_000_000)).expect("valid test fixture");
    let high_id = high.identity();
    state.apply_candidate(DeliveryCandidate {
        post: PostId::new(POST),
        meta: high.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, low],
        discovered_at: 1,
    });
    state.take_representation_bindings();
    learn_validator(&mut state, "v1", 1);
    apply(&mut state, 1, PlayerPreparationState::Initializing, None);
    apply(&mut state, 2, PlayerPreparationState::Failed, Some("decoderUnsupported"));
    assert!(matches!(
        state.planner_capability(&PostId::new(POST), 2),
        PlannerCapability::Reported { playback_supported: false, .. }
    ));

    state.select_capability_fallback(&PostId::new(POST), 2).expect("valid test fixture");
    state
        .catalog_mut()
        .select_rendition_by_representation(&PostId::new(POST), &high_id)
        .expect("valid test fixture");
    learn_validator(&mut state, "v2", 3);

    assert_eq!(
        state.planner_capability(&PostId::new(POST), 3),
        PlannerCapability::Unavailable,
    );
}

fn learn_validator(state: &mut crate::manager::state::DeliveryState, value: &str, at: u64) {
    let post = PostId::new(POST);
    let identity = state.catalog().transfer_identity(&post, SOURCE).expect("valid test fixture");
    let observation = HttpObservation::new(
        LearnedFacts {content_length: Some(16), accept_ranges: Some(true), host: None},
        Some("video/mp4".into()), at,
        EvidenceValidator::strong_etag(format!("\"{value}\"")),
    ).with_final_url(SOURCE);
    assert!(state.catalog_mut().learn_response_observation_for(&identity, observation));
}

fn apply(state: &mut crate::manager::state::DeliveryState, sequence: u64, phase: PlayerPreparationState, failure: Option<&str>) {
    let post = PostId::new(POST);
    let authority = PlayerPreparationAuthority::try_new(
        post.clone(), state.catalog().binding(&post).expect("valid test fixture"), ContentRevision::default(), "asset",
    ).expect("valid test fixture");
    let report = PlayerPreparationReport::try_new(
        authority, PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture"), sequence,
        PlayerPreparationObservation::try_new(phase, failure.map(str::to_owned), sequence).expect("valid test fixture"),
    ).expect("valid test fixture");
    assert!(state.apply_player_preparation(report));
}
