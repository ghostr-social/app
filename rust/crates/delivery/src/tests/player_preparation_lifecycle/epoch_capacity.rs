use crate::delivery_events::{
    command_channel, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn newer_client_epoch_atomically_retires_a_full_active_generation() {
    let (handle, mut receiver) = command_channel();
    let ticket = handle.player_preparation_admission();
    for post in 0..16 {
        assert_eq!(
            handle.report_player_preparation_initial(
                ticket,
                report(post, 7, PlayerPreparationState::Initializing),
            ),
            PlayerPreparationIngress::Accepted,
        );
    }

    assert_eq!(
        handle.report_player_preparation_initial(
            ticket,
            report(99, 8, PlayerPreparationState::Initializing),
        ),
        PlayerPreparationIngress::Accepted,
    );
    let reports: Vec<_> =
        std::iter::from_fn(|| receiver.try_player_preparation()).collect();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].post(), &PostId::new("p99"));
    assert_eq!(reports[0].state(), PlayerPreparationState::Initializing);
    assert_eq!(
        handle.report_player_preparation(report(0, 7, PlayerPreparationState::Released)),
        PlayerPreparationIngress::Stale,
    );
}

fn report(post: u64, epoch: u64, state: PlayerPreparationState) -> PlayerPreparationReport {
    let post_id = PostId::new(format!("p{post}"));
    let binding = Catalog::new().upsert(post_id.clone(), meta());
    let authority = PlayerPreparationAuthority::try_new(
        post_id,
        binding,
        ContentRevision::default(),
        "asset",
    )
    .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(1, epoch, post + 1).unwrap();
    let observation = PlayerPreparationObservation::try_new(state, None, 1).unwrap();
    PlayerPreparationReport::try_new(authority, attempt, 1, observation).unwrap()
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
