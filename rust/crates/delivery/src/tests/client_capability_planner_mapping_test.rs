use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, state, EvidenceSpec};
use ghostr_engine::adaptive::PlannerCapability;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn planner_receives_only_conclusive_versioned_capability_evidence() {
    let mut supported = state(&["current", "next"], 0);
    let post = PostId::new("next");
    let revision = ContentRevision::default();
    assert_eq!(
        supported.planner_capability(&post, 1),
        PlannerCapability::Unavailable,
    );
    apply(
        &mut supported,
        revision,
        100,
        PlayerPreparationState::Initializing,
    );
    assert_eq!(
        supported.planner_capability(&post, 1),
        PlannerCapability::Unavailable,
    );
    apply(
        &mut supported,
        revision,
        350,
        PlayerPreparationState::FirstFrameRendered,
    );
    assert!(matches!(
        supported.planner_capability(&post, 1),
        PlannerCapability::Reported {
            playback_supported: true,
            transform: None,
            evidence_epoch: 2..
        }
    ));

    let mut inconclusive = state(&["current", "next"], 0);
    apply(
        &mut inconclusive,
        revision,
        100,
        PlayerPreparationState::Initializing,
    );
    apply(
        &mut inconclusive,
        revision,
        200,
        PlayerPreparationState::Failed,
    );
    assert_eq!(
        inconclusive.planner_capability(&post, 1),
        PlannerCapability::Unavailable,
    );
}

fn apply(
    state: &mut crate::manager::state::DeliveryState,
    revision: ContentRevision,
    sequence: u64,
    status: PlayerPreparationState,
) {
    assert!(state.apply_player_preparation(evidence(
        state,
        EvidenceSpec {
            post: "next",
            revision,
            sequence,
            state: status,
        },
    )));
}
