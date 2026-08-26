
use crate::delivery_events::{command_channel, PlayerPreparationActorOutcome, PlayerPreparationDisposition};
use crate::tests::player_preparation_fixture::report;

#[tokio::test]
async fn actor_rejection_never_consumes_active_attempt_capacity() {
    let (handle, receiver) = command_channel();
    for generation in 1..=17 {
        let evidence = report(&format!("post-{generation}"), generation, 1);
        let admission = handle.player_preparation_admission();
        let pending = tokio::spawn({
            let handle = handle.clone();
            async move {
                handle
                    .confirm_player_preparation_initial(admission, evidence)
                    .await
            }
        });
        tokio::task::yield_now().await;
        let envelope = receiver.try_player_preparation_envelope().expect("valid test fixture");
        receiver.complete_player_preparation(
            envelope,
            PlayerPreparationActorOutcome::Rejected,
        );
        assert_eq!(pending.await.expect("valid test fixture"), PlayerPreparationDisposition::Rejected);
    }
}
