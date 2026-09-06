use crate::partial_range_store::{PartialRangeStore, StoreAction};
use ghostr_engine::representation::TransferIdentity;

pub(in crate::tests) async fn open(
    store: &PartialRangeStore,
    identity: &TransferIdentity,
    id: u64,
) -> StoreAction {
    let action = store
        .reserve_action(identity, id, 8)
        .await
        .expect("reserve response");
    assert!(matches!(
        store
            .open_action_scoped_single_response(identity, &action, super::exact_response(8))
            .await
            .expect("open response"),
        crate::partial_range_store::ResponseOpenResult::Opened
    ));
    action
}
