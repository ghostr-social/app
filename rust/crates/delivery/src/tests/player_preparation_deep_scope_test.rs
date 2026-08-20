use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, state, EvidenceSpec};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn preparation_scope_tracks_the_probabilistic_ready_target() {
    let revision = ContentRevision::default();
    let mut state = state(&["p0", "p1", "p2", "p3", "p4"], 0);
    state.update_ready_target(3);
    let third = evidence(
        &state,
        EvidenceSpec {
            post: "p3",
            revision,
            sequence: 1,
            state: PlayerPreparationState::FirstFrameRendered,
        },
    );
    assert!(state.apply_player_preparation(third));
    assert!(!state.apply_player_preparation(evidence(
        &state,
        EvidenceSpec {
            post: "p4",
            revision,
            sequence: 1,
            state: PlayerPreparationState::FirstFrameRendered,
        },
    )));

    state.update_ready_target(1);
    assert_eq!(
        state.player_preparation(&PostId::new("p3"), Some(revision)),
        PlayerPreparation::Unverified,
    );
}
