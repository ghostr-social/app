use crate::delivery_events::{
    command_channel, DeliveryHandle, PlayerPreparationActorOutcome, PlayerPreparationAttempt,
    PlayerPreparationAuthority, PlayerPreparationDisposition, PlayerPreparationFollowup,
    PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState,
};
use crate::tests::player_preparation_fixture::report;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[tokio::test]
async fn equal_client_attempt_numbers_remain_scoped_to_their_posts() {
    let (handle, mut receiver) = command_channel();
    let first = confirmation(&handle, report("first", 1, 1));
    tokio::task::yield_now().await;
    apply_next(&mut receiver);
    assert_eq!(first.await.unwrap(), PlayerPreparationDisposition::Applied);

    let second = confirmation(&handle, report("second", 1, 2));
    tokio::task::yield_now().await;
    apply_next(&mut receiver);
    assert_eq!(second.await.unwrap(), PlayerPreparationDisposition::Applied);
}

#[tokio::test]
async fn capability_generation_is_part_of_the_receipt_identity() {
    let (handle, mut receiver) = command_channel();
    let first = confirmation(&handle, capability_report(1));
    tokio::task::yield_now().await;
    apply_next(&mut receiver);
    assert_eq!(first.await.unwrap(), PlayerPreparationDisposition::Applied);

    let collision = PlayerPreparationFollowup::from_report(capability_report(2));
    assert_eq!(handle.player_preparation_disposition(&collision), None);
}

fn confirmation(
    handle: &DeliveryHandle,
    report: PlayerPreparationReport,
) -> tokio::task::JoinHandle<PlayerPreparationDisposition> {
    let admission = handle.player_preparation_admission();
    let handle = handle.clone();
    tokio::spawn(async move {
        handle
            .confirm_player_preparation_initial(admission, report)
            .await
    })
}

fn apply_next(receiver: &mut crate::delivery_events::CommandReceiver) {
    let envelope = receiver
        .try_player_preparation_envelope()
        .expect("independent post attempt");
    receiver.complete_player_preparation(envelope, PlayerPreparationActorOutcome::Applied);
}

fn capability_report(generation: u64) -> PlayerPreparationReport {
    let post = PostId::new("clip");
    let meta = VideoMeta {
        urls: vec!["https://media.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    };
    let binding = Catalog::new().upsert(post.clone(), meta);
    let authority =
        PlayerPreparationAuthority::try_new(post, binding, ContentRevision::default(), "asset")
            .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(generation, 7, 1).unwrap();
    let observation =
        PlayerPreparationObservation::try_new(PlayerPreparationState::Initializing, None, 1)
            .unwrap();
    PlayerPreparationReport::try_new(authority, attempt, 1, observation).unwrap()
}
