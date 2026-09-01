use crate::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use crate::tests::player_preparation_fixture::state;
use crate::transform::{TransformLimits, TransformProfile, TransformTrigger};
use ghostr_engine::adaptive::{PlannerCapability, TransformKind};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn active_transform_suppresses_cross_post_transform_capability() {
    let mut state = state(&["first", "second"], 0);
    state.configure_transform(Some(profile()));
    report_unsupported(&mut state, "second");
    assert!(matches!(
        state.planner_capability(&PostId::new("second"), 1),
        PlannerCapability::Reported {
            transform: Some(_),
            ..
        }
    ));

    assert!(state.begin_transform(PostId::new("first")));
    assert!(matches!(
        state.planner_capability(&PostId::new("second"), 1),
        PlannerCapability::Reported {
            playback_supported: false,
            transform: None,
            ..
        }
    ));
}

fn report_unsupported(state: &mut crate::manager::state::DeliveryState, id: &str) {
    let post = PostId::new(id);
    let authority = PlayerPreparationAuthority::try_new(
        post.clone(),
        state.catalog().binding(&post).expect("valid test fixture"),
        ContentRevision::default(),
        format!("asset-{id}"),
    )
    .expect("valid test fixture");
    let attempt = PlayerPreparationAttempt::try_new(1, 1, 1).expect("valid test fixture");
    apply(
        state,
        authority.clone(),
        attempt,
        (1, PlayerPreparationState::Initializing),
    );
    apply(
        state,
        authority,
        attempt,
        (2, PlayerPreparationState::Failed),
    );
}

fn apply(
    state: &mut crate::manager::state::DeliveryState,
    authority: PlayerPreparationAuthority,
    attempt: PlayerPreparationAttempt,
    evidence: (u64, PlayerPreparationState),
) {
    let (sequence, status) = evidence;
    let failure =
        (status == PlayerPreparationState::Failed).then(|| "invalidVideoTrack".to_owned());
    let observation = PlayerPreparationObservation::try_new(status, failure, sequence)
        .expect("valid test fixture");
    let report = PlayerPreparationReport::try_new(authority, attempt, sequence, observation)
        .expect("valid test fixture");
    assert!(state.apply_player_preparation(report));
}

fn profile() -> TransformProfile {
    TransformProfile::new(
        TransformKind::Remux,
        TransformLimits::try_new(16, 16, 5, 10).expect("valid test fixture"),
    )
    .with_trigger(TransformTrigger::InvalidVideoTrack)
}
