use crate::delivery_events::axiom_test_support::DECODER_UNSUPPORTED_FAILURE;
use crate::delivery_events::{
    DeliveryCandidate, PlayerPreparationActorOutcome, PlayerPreparationAttempt,
    PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use crate::tests::player_preparation_fixture::{meta, state};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn inactive_decoder_rejection_selects_missing_bitrate_alternative() {
    let mut state = state(&["current", "next"], 0);
    let post = PostId::new("next");
    let high = rendition("hevc");
    let fallback = rendition("avc");
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: high.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, fallback],
        discovered_at: 1,
    });
    state.take_representation_bindings();
    let old = state.catalog().binding(&post).expect("valid test fixture");
    let attempt = PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture");
    assert!(state.apply_player_preparation(report(
        old.clone(),
        attempt,
        1,
        PlayerPreparationState::Initializing,
        None,
    )));
    assert!(state.apply_player_preparation(report(
        old.clone(),
        attempt,
        2,
        PlayerPreparationState::Failed,
        Some(DECODER_UNSUPPORTED_FAILURE),
    )));

    let selected = state
        .select_capability_fallback(&post, 0)
        .expect("valid test fixture");

    assert_ne!(selected.representation(), old.representation());
    assert_eq!(
        state
            .catalog()
            .lookup(&post)
            .expect("valid test fixture")
            .meta,
        meta("avc")
    );
    assert_eq!(
        state.apply_player_preparation_at(
            report(
                old,
                attempt,
                3,
                PlayerPreparationState::FirstFrameRendered,
                None,
            ),
            1,
        ),
        PlayerPreparationActorOutcome::Rejected,
    );
}

fn rendition(id: &str) -> VideoRendition {
    VideoRendition::try_new(meta(id), None).expect("valid test fixture")
}

fn report(
    binding: ghostr_engine::representation::RepresentationBinding,
    attempt: PlayerPreparationAttempt,
    sequence: u64,
    state: PlayerPreparationState,
    failure: Option<&str>,
) -> PlayerPreparationReport {
    let authority = PlayerPreparationAuthority::try_new(
        PostId::new("next"),
        binding,
        ContentRevision::default(),
        "asset-next",
    )
    .expect("valid test fixture");
    let observation =
        PlayerPreparationObservation::try_new(state, failure.map(str::to_owned), sequence)
            .expect("valid test fixture");
    PlayerPreparationReport::try_new(authority, attempt, sequence, observation)
        .expect("valid test fixture")
}
