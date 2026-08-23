use ghostr_partial_store::partial_range_store::capacity::CapacityRevision;
use std::time::Duration;
use tokio::sync::watch;

pub(crate) async fn capacity_changed(
    store: &ghostr_partial_store::partial_range_store::PartialRangeStore,
    changes: &mut watch::Receiver<u64>,
    recheck_after: Duration,
    observed: CapacityRevision,
) -> bool {
    if *changes.borrow_and_update() != observed.value() {
        return true;
    }
    tokio::select! {
        result = changes.changed() => result.is_ok(),
        _ = tokio::time::sleep(recheck_after) => {
            store.recheck_capacity().await;
            changes.changed().await.is_ok()
        }
    }
}
