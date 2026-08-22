use crate::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use crate::tests::player_preparation_fixture::state;
use ghostr_engine::adaptive::PlannerCapability;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn absent_measured_backend_never_advertises_transform_rescue() {
    let mut state = state(&["current", "next"], 0);
    state.configure_transform(None);
    report(&mut state, 1, PlayerPreparationState::Initializing, None);
    report(
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

fn report(
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
        PlayerPreparationObservation::try_new(status, failure.map(str::to_owned), sequence)
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
