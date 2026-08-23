use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, state, EvidenceSpec};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn state_accepts_only_the_planning_window_and_rechecks_all_authority() {
    let owned: Vec<_> = (0..=25).map(|index| format!("p{index}")).collect();
    let ids: Vec<_> = owned.iter().map(String::as_str).collect();
    let mut state = state(&ids, 0);
    let revision = ContentRevision::default();
    let next = evidence(
        &state,
        EvidenceSpec {
            post: "p1",
            revision,
            sequence: 2,
            state: PlayerPreparationState::FirstFrameRendered,
        },
    );
    assert!(state.apply_player_preparation(next.clone()));
    assert!(!state.apply_player_preparation(evidence(
        &state,
        EvidenceSpec {
            post: "p25",
            revision,
            sequence: 2,
            state: PlayerPreparationState::FirstFrameRendered,
        }
    )));
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::FirstFrameRendered
    );
    assert!(!state.apply_player_preparation(evidence(
        &state,
        EvidenceSpec {
            post: "p1",
            revision,
            sequence: 1,
            state: PlayerPreparationState::Failed,
        }
    )));
    assert!(state.apply_player_preparation(evidence(
        &state,
        EvidenceSpec {
            post: "p1",
            revision,
            sequence: 3,
            state: PlayerPreparationState::Released,
        }
    )));
    assert_eq!(
        state.player_preparation(&PostId::new("p1"), Some(revision)),
        PlayerPreparation::Unverified
    );

    assert!(!state.apply_player_preparation(next));
}
