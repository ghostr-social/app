use crate::delivery_events::{
    command_channel, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn client_epoch_keeps_one_capability_generation_after_terminal_cleanup() {
    let (handle, mut receiver) = command_channel();
    let ticket = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(ticket, report(1, 7, 1, 1)),
        PlayerPreparationIngress::Accepted,
    );
    assert_eq!(
        handle.report_player_preparation(report(1, 7, 1, 2)),
        PlayerPreparationIngress::Accepted,
    );
    while receiver.try_player_preparation().is_some() {}

    let ticket = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(ticket, report(2, 7, 2, 1)),
        PlayerPreparationIngress::Rejected,
    );
    assert_eq!(
        handle.report_player_preparation_initial(ticket, report(2, 8, 2, 1)),
        PlayerPreparationIngress::Accepted,
    );
}

fn report(generation: u64, epoch: u64, attempt: u64, sequence: u64) -> PlayerPreparationReport {
    let post = PostId::new("p1");
    let binding = Catalog::new().upsert(post.clone(), meta());
    let authority = PlayerPreparationAuthority::try_new(
        post,
        binding,
        ContentRevision::default(),
        "asset",
    )
    .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(generation, epoch, attempt).unwrap();
    let state = if sequence == 1 {
        PlayerPreparationState::Initializing
    } else {
        PlayerPreparationState::Released
    };
    let observation = PlayerPreparationObservation::try_new(state, None, sequence).unwrap();
    PlayerPreparationReport::try_new(authority, attempt, sequence, observation).unwrap()
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
