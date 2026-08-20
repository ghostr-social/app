use crate::client_capability::ClientCapabilityStatus;
use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, state, EvidenceSpec};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn accepted_first_frame_updates_the_versioned_client_profile() {
    let mut state = state(&["current", "next"], 0);
    let revision = ContentRevision::default();
    let next = PostId::new("next");
    assert!(state.apply_player_preparation(evidence(
        &state,
        EvidenceSpec {
            post: "next",
            revision,
            sequence: 100,
            state: PlayerPreparationState::Initializing,
        },
    )));
    assert_eq!(
        state.client_capability_status(&next, 1, 1),
        ClientCapabilityStatus::Testing,
    );

    assert!(state.apply_player_preparation(evidence(
        &state,
        EvidenceSpec {
            post: "next",
            revision,
            sequence: 350,
            state: PlayerPreparationState::FirstFrameRendered,
        },
    )));
    assert_eq!(
        state.client_capability_status(&next, 1, 1),
        ClientCapabilityStatus::Supported {
            p95_first_frame_us: 250,
        },
    );
}
