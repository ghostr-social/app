use crate::delivery_events::{
    command_channel, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationFollowup, PlayerPreparationIngress, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn acknowledged_initial_can_be_replayed_after_manager_reset() {
    let (handle, mut receiver) = command_channel();
    let initial = report(1, PlayerPreparationState::Initializing);
    let followup = PlayerPreparationFollowup::from_report(report(
        2,
        PlayerPreparationState::Initialized,
    ));
    let admission = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(admission, initial.clone()),
        PlayerPreparationIngress::Accepted,
    );
    receiver.try_player_preparation().unwrap();
    receiver.discard_pending();

    assert_eq!(
        handle.report_player_preparation_followup(followup.clone()),
        PlayerPreparationIngress::MissingInitial,
    );
    let admission = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(admission, initial),
        PlayerPreparationIngress::Accepted,
    );
    receiver.try_player_preparation().unwrap();
    assert_eq!(
        handle.report_player_preparation_followup(followup),
        PlayerPreparationIngress::Accepted,
    );
    assert_eq!(
        receiver.try_player_preparation().unwrap().state(),
        PlayerPreparationState::Initialized,
    );
}

fn report(sequence: u64, state: PlayerPreparationState) -> PlayerPreparationReport {
    let post = PostId::new("clip");
    let binding = Catalog::new().upsert(post.clone(), meta());
    let authority =
        PlayerPreparationAuthority::try_new(post, binding, ContentRevision::default(), "asset")
            .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(1, 7, 1).unwrap();
    let observation = PlayerPreparationObservation::try_new(state, None, sequence).unwrap();
    PlayerPreparationReport::try_new(authority, attempt, sequence, observation).unwrap()
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
