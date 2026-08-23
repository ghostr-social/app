use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, state, EvidenceSpec};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::HashMap;

#[test]
fn missing_content_revision_retains_live_player_evidence_until_terminal() {
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

    state.prune_player_preparations(&HashMap::new());

    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::FirstFrameRendered,
    );
}
