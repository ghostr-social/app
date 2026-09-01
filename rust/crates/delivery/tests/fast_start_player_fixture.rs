use core::time::Duration;
use ghostr_delivery::delivery_events::*;
use PlayerPreparationState as State;

pub async fn report_failed(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    generation: u64,
) {
    report(handle, authority, generation, State::Failed).await;
}

pub async fn report_ready(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    generation: u64,
) {
    report(handle, authority, generation, State::FirstFrameRendered).await;
}

pub async fn report_rejected(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    generation: u64,
) {
    let attempt =
        PlayerPreparationAttempt::try_new(generation, generation, 1).expect("valid test fixture");
    let disposition = send(handle, authority, attempt, (1, State::Initializing)).await;
    assert_eq!(disposition, PlayerPreparationDisposition::Rejected);
}

async fn report(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    generation: u64,
    terminal: State,
) {
    let attempt =
        PlayerPreparationAttempt::try_new(generation, generation, 1).expect("valid test fixture");
    assert_applied(send(handle, authority.clone(), attempt, (1, State::Initializing)).await);
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_applied(send(handle, authority, attempt, (2, terminal)).await);
}

fn assert_applied(disposition: PlayerPreparationDisposition) {
    assert_eq!(disposition, PlayerPreparationDisposition::Applied);
}

async fn send(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    attempt: PlayerPreparationAttempt,
    evidence: (u64, State),
) -> PlayerPreparationDisposition {
    let (sequence, state) = evidence;
    let failure = (state == State::Failed).then(|| "invalidVideoTrack".to_owned());
    let observation = PlayerPreparationObservation::try_new(state, failure, sequence * 100)
        .expect("valid test fixture");
    let report =
        PlayerPreparationReport::try_new(authority, attempt, sequence, observation.clone())
            .expect("valid test fixture");
    if sequence == 1 {
        let admission = handle.player_preparation_admission();
        handle
            .confirm_player_preparation_initial(admission, report)
            .await
    } else {
        let claim = PlayerPreparationClaim::try_new(
            report.post().clone(),
            report
                .progressive_binding()
                .expect("progressive authority")
                .representation()
                .fingerprint(),
            "asset",
        )
        .expect("valid test fixture");
        let followup = PlayerPreparationFollowup::try_new(claim, attempt, sequence, observation)
            .expect("valid test fixture");
        handle.confirm_player_preparation_followup(followup).await
    }
}
