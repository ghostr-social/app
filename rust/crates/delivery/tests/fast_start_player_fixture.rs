use ghostr_delivery::delivery_events::*;
use std::time::Duration;

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
    let attempt = PlayerPreparationAttempt::try_new(generation, generation, 1).unwrap();
    send(
        handle,
        authority.clone(),
        attempt,
        (1, PlayerPreparationState::Initializing),
    );
    tokio::time::sleep(Duration::from_millis(20)).await;
    send(handle, authority, attempt, (2, terminal));
}

fn send(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    attempt: PlayerPreparationAttempt,
    evidence: (u64, PlayerPreparationState),
) {
    let (sequence, state) = evidence;
    let failure = (state == PlayerPreparationState::Failed).then(|| "invalidVideoTrack".to_owned());
    let observation =
        PlayerPreparationObservation::try_new(state, failure, sequence * 100).unwrap();
    let report =
        PlayerPreparationReport::try_new(authority, attempt, sequence, observation).unwrap();
    handle.report_player_preparation(report);
}
