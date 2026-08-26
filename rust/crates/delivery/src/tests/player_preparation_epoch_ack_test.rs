
use crate::delivery_events::{command_channel, PlayerPreparationActorOutcome, PlayerPreparationDisposition};
use crate::tests::player_preparation_fixture::report_with_epoch;

#[tokio::test]
async fn old_completion_cannot_mutate_a_new_epoch_ledger() {
    let (handle, receiver) = command_channel();
    let old = report_with_epoch("old", 7, 1, 1);
    let old_pending = confirmation(&handle, old);
    tokio::task::yield_now().await;
    let old_envelope = receiver.try_player_preparation_envelope().expect("valid test fixture");

    let new = report_with_epoch("new", 8, 1, 1);
    let new_pending = confirmation(&handle, new.clone());
    tokio::task::yield_now().await;
    let new_envelope = receiver.try_player_preparation_envelope().expect("valid test fixture");
    receiver.complete_player_preparation(old_envelope, PlayerPreparationActorOutcome::Applied);
    assert_eq!(old_pending.await.expect("valid test fixture"), PlayerPreparationDisposition::Unavailable);
    receiver.complete_player_preparation(new_envelope, PlayerPreparationActorOutcome::Applied);
    assert_eq!(new_pending.await.expect("valid test fixture"), PlayerPreparationDisposition::Applied);

    assert_eq!(
        confirmation(&handle, new).await.expect("valid test fixture"),
        PlayerPreparationDisposition::Duplicate,
    );
}

fn confirmation(
    handle: &crate::delivery_events::DeliveryHandle,
    evidence: crate::delivery_events::PlayerPreparationReport,
) -> tokio::task::JoinHandle<PlayerPreparationDisposition> {
    let admission = handle.player_preparation_admission();
    let handle = handle.clone();
    tokio::spawn(async move {
        handle
            .confirm_player_preparation_initial(admission, evidence)
            .await
    })
}
