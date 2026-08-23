use crate::client_capability::ClientCapabilityStatus;
use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, focus, state, EvidenceSpec};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn scope_and_reset_abandon_active_capability_tests() {
    let mut state = state(&["p0", "p1", "p2"], 0);
    let post = PostId::new("p1");
    initialize(&mut state);
    assert_eq!(
        state.client_capability_status(&post, 1, 1),
        ClientCapabilityStatus::Testing,
    );

    state.apply_focus(focus(&["p0", "p1", "p2"], 2), 2);
    assert_eq!(
        state.client_capability_status(&post, 1, 1),
        ClientCapabilityStatus::Unknown,
    );

    state.apply_focus(focus(&["p0", "p1", "p2"], 0), 3);
    initialize(&mut state);
    state.clear();
    state.apply_focus(focus(&["p0", "p1", "p2"], 0), 4);
    assert_eq!(
        state.client_capability_status(&post, 1, 1),
        ClientCapabilityStatus::Unknown,
    );
}

fn initialize(state: &mut crate::manager::state::DeliveryState) {
    let report = evidence(
        state,
        EvidenceSpec {
            post: "p1",
            revision: ContentRevision::default(),
            sequence: 1,
            state: PlayerPreparationState::Initializing,
        },
    );
    assert!(state.apply_player_preparation(report));
}
