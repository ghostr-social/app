use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, focus, state, EvidenceSpec};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn adjacent_preparation_survives_promotion_and_drops_after_scroll_past() {
    let revision = ContentRevision::default();
    let mut state = state(&["p0", "p1", "p2"], 0);
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

    state.apply_focus(focus(&["p0", "p1", "p2"], 1), 2);
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::FirstFrameRendered,
    );

    state.apply_focus(focus(&["p0", "p1", "p2"], 2), 3);
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::Unverified,
    );
}
