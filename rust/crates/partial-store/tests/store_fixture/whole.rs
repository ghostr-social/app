use crate::partial_range_store::PartialRangeStore;
use ghostr_engine::representation::TransferIdentity;

pub(super) async fn publish_whole(
    store: &PartialRangeStore,
    identity: &TransferIdentity,
    id: u64,
    bytes: &[u8],
) {
    let total = bytes.len() as u64;
    let action = store
        .reserve_action(identity, id, total)
        .await
        .expect("valid test fixture");
    assert!(
        store
            .begin_single_response_for_action(identity, &action, super::exact_response(total))
            .await
            .expect("valid test fixture"),
        "whole response should open"
    );
    assert!(
        store
            .write_single_response_for_action(identity, &action, 0, bytes)
            .await
            .expect("valid test fixture"),
        "whole response bytes should remain current"
    );
    assert!(
        store
            .finish_single_response_for_action(identity, &action, Some(total), true)
            .await
            .expect("valid test fixture"),
        "whole response should finish"
    );
    store.release_action(&action).await;
}
