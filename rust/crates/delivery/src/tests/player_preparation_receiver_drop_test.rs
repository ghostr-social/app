
use crate::delivery_events::{command_channel, PlayerPreparationDisposition};
use crate::tests::player_preparation_fixture::report;

#[tokio::test]
async fn manager_loss_releases_queued_and_popped_confirmations() {
    let (handle, receiver) = command_channel();
    let queued = confirmation(&handle, report("queued", 1, 1));
    tokio::task::yield_now().await;
    drop(receiver);
    assert_eq!(queued.await.expect("valid test fixture"), PlayerPreparationDisposition::Unavailable);

    let (handle, receiver) = command_channel();
    let popped = confirmation(&handle, report("popped", 1, 1));
    tokio::task::yield_now().await;
    let envelope = receiver.try_player_preparation_envelope().expect("valid test fixture");
    drop(receiver);
    drop(envelope);
    assert_eq!(popped.await.expect("valid test fixture"), PlayerPreparationDisposition::Unavailable);
}

#[tokio::test]
async fn retry_after_manager_loss_is_terminally_closed() {
    let (handle, receiver) = command_channel();
    let evidence = report("queued", 1, 1);
    let queued = confirmation(&handle, evidence.clone());
    tokio::task::yield_now().await;
    drop(receiver);
    assert_eq!(queued.await.expect("valid test fixture"), PlayerPreparationDisposition::Unavailable);

    let admission = handle.player_preparation_admission();
    assert_eq!(
        handle
            .confirm_player_preparation_initial(admission, evidence)
            .await,
        PlayerPreparationDisposition::Closed,
    );
}

fn confirmation(
    handle: &crate::delivery_events::DeliveryHandle,
    report: crate::delivery_events::PlayerPreparationReport,
) -> tokio::task::JoinHandle<PlayerPreparationDisposition> {
    let admission = handle.player_preparation_admission();
    let handle = handle.clone();
    tokio::spawn(async move {
        handle
            .confirm_player_preparation_initial(admission, report)
            .await
    })
}
