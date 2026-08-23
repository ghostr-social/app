use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, focus, state, EvidenceSpec};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn nearby_preparation_survives_backward_swipes_and_drops_outside_planning_window() {
    let revision = ContentRevision::default();
    let ids = ["p0", "p1", "p2", "p3", "p4", "p5"];
    let mut state = state(&ids, 0);
    let report = evidence(
        &state,
        EvidenceSpec {
            post: "p1",
            revision,
            sequence: 1,
            state: PlayerPreparationState::FirstFrameRendered,
        },
    );
    assert!(state.apply_player_preparation(report));

    state.apply_focus(focus(&ids, 2), 2);
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::FirstFrameRendered,
    );

    state.apply_focus(focus(&ids, 5), 3);
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::Unverified,
    );
}

#[test]
fn delayed_player_feedback_survives_a_rapid_forward_swipe() {
    let revision = ContentRevision::default();
    let ids = ["p0", "p1", "p2"];
    let mut state = state(&ids, 0);
    let initializing = evidence(
        &state,
        EvidenceSpec {
            post: "p1",
            revision,
            sequence: 1,
            state: PlayerPreparationState::Initializing,
        },
    );
    assert!(state.apply_player_preparation(initializing));

    state.apply_focus(focus(&ids, 2), 2);
    for (sequence, player_state) in [
        (2, PlayerPreparationState::Initialized),
        (3, PlayerPreparationState::FirstFrameRendered),
    ] {
        let report = evidence(
            &state,
            EvidenceSpec {
                post: "p1",
                revision,
                sequence,
                state: player_state,
            },
        );
        assert!(state.apply_player_preparation(report));
    }
    state.apply_focus(focus(&ids, 1), 3);
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::FirstFrameRendered,
    );
}
