use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, focus, state, EvidenceSpec};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn representation_change_and_clear_remove_player_evidence() {
    let revision = ContentRevision::default();
    let mut state = state(&["p0", "p1"], 0);
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

    let mut changed = focus(&["p0", "p1"], 0);
    changed.items[1].meta.urls = vec!["https://replacement.example/p1.mp4".to_owned()];
    state.apply_focus(changed, 2);
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::Unverified,
    );

    let replacement = evidence(
        &state,
        EvidenceSpec {
            post: "p1",
            revision,
            sequence: 2,
            state: PlayerPreparationState::Initialized,
        },
    );
    assert!(state.apply_player_preparation(replacement));
    state.clear();
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::Unverified,
    );
    state.apply_focus(focus(&["p0", "p1"], 0), 3);
    let after_clear = evidence(
        &state,
        EvidenceSpec {
            post: "p1",
            revision,
            sequence: 2,
            state: PlayerPreparationState::Initialized,
        },
    );
    assert!(state.apply_player_preparation(after_clear));
}
