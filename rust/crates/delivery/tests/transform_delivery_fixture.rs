use core::time::Duration;
use ghostr_delivery::delivery_events::*;
use ghostr_engine::{catalog::Catalog, representation::RepresentationBinding};
use ghostr_partial_store::partial_range_store::PartialRangeStore;

pub async fn seed_input(
    store: &PartialRangeStore,
    item: &FocusItem,
    bytes: &[u8],
) -> RepresentationBinding {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .set_total_len("post", bytes.len() as u64)
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, bytes)
        .await
        .expect("valid test fixture");
    store
        .finalize("post", None)
        .await
        .expect("valid test fixture");
    binding
}

pub async fn report_unsupported(
    handle: &DeliveryHandle,
    store: &PartialRangeStore,
    binding: RepresentationBinding,
) {
    let revision = store
        .media_snapshot("post")
        .await
        .expect("valid test fixture")
        .revision();
    let authority = PlayerPreparationAuthority::try_new(
        ghostr_engine::PostId::new("post"),
        binding,
        revision,
        "asset",
    )
    .expect("valid test fixture");
    let attempt = PlayerPreparationAttempt::try_new(1, 1, 1).expect("valid test fixture");
    send(
        handle,
        authority.clone(),
        attempt,
        (1, PlayerPreparationState::Initializing, None),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(20)).await;
    let failure = (
        2,
        PlayerPreparationState::Failed,
        Some("invalidVideoTrack".into()),
    );
    send(handle, authority, attempt, failure).await;
}

async fn send(
    handle: &DeliveryHandle,
    authority: PlayerPreparationAuthority,
    attempt: PlayerPreparationAttempt,
    evidence: (u64, PlayerPreparationState, Option<String>),
) {
    let (sequence, state, failure) = evidence;
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
        let binding = report.progressive_binding().expect("progressive authority");
        let claim = PlayerPreparationClaim::try_new(
            report.post().clone(),
            binding.representation().fingerprint(),
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
