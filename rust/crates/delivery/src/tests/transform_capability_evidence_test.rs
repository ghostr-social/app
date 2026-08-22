use crate::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use crate::tests::player_preparation_fixture::state;
use crate::transform::{FastStartRemuxBackend, TransformBackend};
use ghostr_engine::adaptive::PlannerCapability;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn generic_unsupported_video_track_does_not_advertise_fast_start_remux() {
    let mut state = state(&["current", "next"], 0);
    state.configure_transform(Some(FastStartRemuxBackend::production().profile()));
    apply(&mut state, 1, PlayerPreparationState::Initializing, None);
    apply(
        &mut state,
        2,
        PlayerPreparationState::Failed,
        Some("invalidVideoTrack"),
    );

    assert!(matches!(
        state.planner_capability(&PostId::new("next"), 1),
        PlannerCapability::Reported {
            playback_supported: false,
            transform: None,
            ..
        }
    ));
}

fn apply(
    state: &mut crate::manager::state::DeliveryState,
    sequence: u64,
    status: PlayerPreparationState,
    failure: Option<&str>,
) {
    let post = PostId::new("next");
    let authority = PlayerPreparationAuthority::try_new(
        post.clone(),
        state.catalog().binding(&post).unwrap(),
        ContentRevision::default(),
    )
    .unwrap();
    let observation =
        PlayerPreparationObservation::try_new(status, failure.map(str::to_owned), sequence * 100)
            .unwrap();
    let report = PlayerPreparationReport::try_new(
        authority,
        PlayerPreparationAttempt::try_new(1, 1, 1).unwrap(),
        sequence,
        observation,
    )
    .unwrap();
    assert!(state.apply_player_preparation(report));
}
