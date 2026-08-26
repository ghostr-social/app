
use crate::delivery_events::{DeliveryCandidate, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState, DECODER_UNSUPPORTED_FAILURE};
use crate::tests::player_preparation_fixture::{meta, state};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn decoder_rejection_after_first_frame_selects_a_fallback_rendition() {
    let mut state = state(&["current", "next"], 0);
    let post = PostId::new("next");
    let high = VideoRendition::try_new(meta("high"), Some(6_000_000)).expect("valid test fixture");
    let low = VideoRendition::try_new(meta("low"), Some(1_000_000)).expect("valid test fixture");
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: high.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, low],
        discovered_at: 1,
    });
    state.take_representation_bindings();
    apply(&mut state, 1, PlayerPreparationState::Initializing, None);
    apply(
        &mut state,
        2,
        PlayerPreparationState::FirstFrameRendered,
        None,
    );
    apply(
        &mut state,
        3,
        PlayerPreparationState::Failed,
        Some(DECODER_UNSUPPORTED_FAILURE),
    );

    state.select_capability_fallback(&post, 3).expect("valid test fixture");

    assert_eq!(state.catalog().lookup(&post).expect("valid test fixture").meta, meta("low"));
}

fn apply(
    state: &mut crate::manager::state::DeliveryState,
    sequence: u64,
    phase: PlayerPreparationState,
    failure: Option<&str>,
) {
    let post = PostId::new("next");
    let authority = PlayerPreparationAuthority::try_new(
        post,
        state.catalog().binding(&PostId::new("next")).expect("valid test fixture"),
        ContentRevision::default(),
        "asset",
    )
    .expect("valid test fixture");
    let report = PlayerPreparationReport::try_new(
        authority,
        PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture"),
        sequence,
        PlayerPreparationObservation::try_new(phase, failure.map(str::to_owned), sequence).expect("valid test fixture"),
    )
    .expect("valid test fixture");
    assert!(state.apply_player_preparation(report));
}
