use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, state, EvidenceSpec};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn preparation_scope_survives_beyond_the_ready_target() {
    let revision = ContentRevision::default();
    let owned: Vec<_> = (0..=3).map(|index| format!("p{index}")).collect();
    let ids: Vec<_> = owned.iter().map(String::as_str).collect();
    let mut state = state(&ids, 0);
    state.update_ready_target(1);
    let nearby = evidence(
        &state,
        EvidenceSpec {
            post: "p3",
            revision,
            sequence: 1,
            state: PlayerPreparationState::Initializing,
        },
    );
    assert!(state.apply_player_preparation(nearby));

    assert_eq!(
        state.player_preparation(&PostId::new("p3"), Some(revision)),
        PlayerPreparation::Initializing,
    );
}
