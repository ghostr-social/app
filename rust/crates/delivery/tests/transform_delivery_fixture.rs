use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::time::Duration;

pub async fn seed_input(
    store: &PartialRangeStore,
    item: &ghostr_delivery::delivery_events::FocusItem,
    bytes: &[u8],
) -> RepresentationBinding {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    store.bind_representation(binding.clone()).await.unwrap();
    store
        .set_total_len("post", bytes.len() as u64)
        .await
        .unwrap();
    store.write_range("post", 0, bytes).await.unwrap();
    store.finalize("post", None).await.unwrap();
    binding
}

pub async fn report_unsupported(
    handle: &ghostr_delivery::delivery_events::DeliveryHandle,
    store: &PartialRangeStore,
    binding: RepresentationBinding,
) {
    use ghostr_delivery::delivery_events::*;
    let revision = store.media_snapshot("post").await.unwrap().revision();
    let authority =
        PlayerPreparationAuthority::try_new(ghostr_engine::PostId::new("post"), binding, revision)
            .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(1, 1, 1).unwrap();
    send(
        handle,
        authority.clone(),
        attempt,
        (1, PlayerPreparationState::Initializing, None),
    );
    tokio::time::sleep(Duration::from_millis(20)).await;
    send(
        handle,
        authority,
        attempt,
        (
            2,
            PlayerPreparationState::Failed,
            Some("invalidVideoTrack".into()),
        ),
    );
}

fn send(
    handle: &ghostr_delivery::delivery_events::DeliveryHandle,
    authority: ghostr_delivery::delivery_events::PlayerPreparationAuthority,
    attempt: ghostr_delivery::delivery_events::PlayerPreparationAttempt,
    evidence: (
        u64,
        ghostr_delivery::delivery_events::PlayerPreparationState,
        Option<String>,
    ),
) {
    use ghostr_delivery::delivery_events::*;
    let (sequence, state, failure) = evidence;
    let observation =
        PlayerPreparationObservation::try_new(state, failure, sequence * 100).unwrap();
    let report =
        PlayerPreparationReport::try_new(authority, attempt, sequence, observation).unwrap();
    handle.report_player_preparation(report);
}
