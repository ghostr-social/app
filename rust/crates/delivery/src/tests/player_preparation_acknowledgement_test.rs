
use crate::delivery_events::{command_channel, PlayerPreparationActorOutcome, PlayerPreparationDisposition};
use crate::tests::player_preparation_fixture::report;

#[tokio::test]
async fn acknowledgement_waits_for_the_actor_and_is_idempotent() {
    let (handle, receiver) = command_channel();
    let initial = report("first", 1, 1);
    let ticket = handle.player_preparation_admission();
    let pending = tokio::spawn({
        let handle = handle.clone();
        let report = initial.clone();
        async move { handle.confirm_player_preparation_initial(ticket, report).await }
    });
    tokio::task::yield_now().await;
    let envelope = receiver.try_player_preparation_envelope().expect("valid test fixture");
    assert!(!pending.is_finished());
    receiver.complete_player_preparation(envelope, PlayerPreparationActorOutcome::Applied);
    assert_eq!(pending.await.expect("valid test fixture"), PlayerPreparationDisposition::Applied);

    let ticket = handle.player_preparation_admission();
    assert_eq!(
        handle
            .confirm_player_preparation_initial(ticket, initial)
            .await,
        PlayerPreparationDisposition::Duplicate,
    );
}

#[tokio::test]
async fn conflicting_replay_is_rejected_and_clear_releases_pending_confirmation() {
    let (handle, receiver) = command_channel();
    let initial = report("first", 1, 1);
    let ticket = handle.player_preparation_admission();
    let pending = tokio::spawn({
        let handle = handle.clone();
        let report = initial.clone();
        async move { handle.confirm_player_preparation_initial(ticket, report).await }
    });
    tokio::task::yield_now().await;
    let envelope = receiver.try_player_preparation_envelope().expect("valid test fixture");
    receiver.complete_player_preparation(envelope, PlayerPreparationActorOutcome::Applied);
    assert_eq!(pending.await.expect("valid test fixture"), PlayerPreparationDisposition::Applied);

    let ticket = handle.player_preparation_admission();
    assert_eq!(
        handle
            .confirm_player_preparation_initial(ticket, report("first", 1, 2))
            .await,
        PlayerPreparationDisposition::Rejected,
    );

    let ticket = handle.player_preparation_admission();
    let cleared = tokio::spawn({
        let handle = handle.clone();
        async move {
            handle
                .confirm_player_preparation_initial(ticket, report("second", 2, 1))
                .await
        }
    });
    tokio::task::yield_now().await;
    receiver.discard_pending();
    assert_eq!(cleared.await.expect("valid test fixture"), PlayerPreparationDisposition::Unavailable);
}

#[tokio::test]
async fn invalid_admission_is_proven_not_admitted() {
    let (handle, receiver) = command_channel();
    let stale = handle.player_preparation_admission();
    receiver.discard_pending();

    assert_eq!(
        handle
            .confirm_player_preparation_initial(stale, report("stale", 1, 1))
            .await,
        PlayerPreparationDisposition::NotAdmitted,
    );
    assert!(receiver.try_player_preparation().is_none());
}
