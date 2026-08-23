use ghostr_engine::representation::TransferIdentity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;

pub async fn publish_whole(
    store: &PartialRangeStore,
    identity: &TransferIdentity,
    id: u64,
    bytes: &[u8],
) {
    let total = bytes.len() as u64;
    let action = store.reserve_action(identity, id, total).await.unwrap();
    assert!(store
        .begin_single_response_for_action(identity, &action, super::exact_response(total))
        .await
        .unwrap());
    assert!(store
        .write_single_response_for_action(identity, &action, 0, bytes)
        .await
        .unwrap());
    assert!(store
        .finish_single_response_for_action(identity, &action, Some(total), true)
        .await
        .unwrap());
    store.release_action(&action).await;
}
