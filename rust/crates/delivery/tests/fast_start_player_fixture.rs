use core::time::Duration;
use ghostr_delivery::delivery_events::*;

pub async fn report_failed(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    generation: u64,
) {
    report(
        handle,
        authority,
        generation,
        PlayerPreparationState::Failed,
    )
    .await;
}

pub async fn report_ready(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    generation: u64,
) {
    report(
        handle,
        authority,
        generation,
        PlayerPreparationState::FirstFrameRendered,
    )
    .await;
}

async fn report(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    generation: u64,
    terminal: PlayerPreparationState,
) {
    let attempt =
        PlayerPreparationAttempt::try_new(generation, generation, 1).expect("valid test fixture");
    send(
        handle,
        authority.clone(),
        attempt,
        (1, PlayerPreparationState::Initializing),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(20)).await;
    send(handle, authority, attempt, (2, terminal)).await;
}

async fn send(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    attempt: PlayerPreparationAttempt,
    evidence: (u64, PlayerPreparationState),
) {
    let (sequence, state) = evidence;
    let failure = (state == PlayerPreparationState::Failed).then(|| "invalidVideoTrack".to_owned());
    let observation = PlayerPreparationObservation::try_new(state, failure, sequence * 100)
        .expect("valid test fixture");
    let report =
        PlayerPreparationReport::try_new(authority, attempt, sequence, observation.clone())
            .expect("valid test fixture");
    let disposition = if sequence == 1 {
        let admission = handle.player_preparation_admission();
        handle
            .confirm_player_preparation_initial(admission, report)
            .await
    } else {
        let claim = PlayerPreparationClaim::try_new(
            report.post().clone(),
            report.binding().representation().fingerprint(),
            "asset",
        )
        .expect("valid test fixture");
        let followup = PlayerPreparationFollowup::try_new(claim, attempt, sequence, observation)
            .expect("valid test fixture");
        handle.confirm_player_preparation_followup(followup).await
    };
    assert_eq!(
        disposition,
        PlayerPreparationDisposition::Applied,
        "fixture preparation should be admitted"
    );
}
