use crate::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use crate::tests::player_preparation_fixture::state;
use ghostr_engine::adaptive::{PlannerCapability, TransformKind};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::HashMap;

#[test]
fn transformed_bytes_are_not_declared_playable_without_fresh_player_evidence() {
    let mut state = state(&["current", "next"], 0);
    let post = PostId::new("next");
    let input = state.catalog().binding(&post).unwrap();
    let derived = input
        .derive_transform(TransformKind::Remux, &"ab".repeat(32))
        .unwrap();
    state.replace_transformed_posts(HashMap::from([(post.clone(), derived.clone())]));

    assert_eq!(
        state.planner_capability(&post, 1),
        PlannerCapability::Unavailable
    );
    apply(
        &mut state,
        &derived,
        1,
        PlayerPreparationState::Initializing,
    );
    apply(
        &mut state,
        &derived,
        2,
        PlayerPreparationState::FirstFrameRendered,
    );
    assert!(matches!(
        state.planner_capability(&post, 1),
        PlannerCapability::Reported {
            playback_supported: true,
            transform: None,
            ..
        }
    ));
}

fn apply(
    state: &mut crate::manager::state::DeliveryState,
    binding: &RepresentationBinding,
    sequence: u64,
    status: PlayerPreparationState,
) {
    let authority = PlayerPreparationAuthority::try_new(
        binding.post().clone(),
        binding.clone(),
        ContentRevision::default(),
        "asset",
    )
    .unwrap();
    let observation = PlayerPreparationObservation::try_new(status, None, sequence * 100).unwrap();
    let report = PlayerPreparationReport::try_new(
        authority,
        PlayerPreparationAttempt::try_new(1, 1, 1).unwrap(),
        sequence,
        observation,
    )
    .unwrap();
    assert!(state.apply_player_preparation(report));
}
