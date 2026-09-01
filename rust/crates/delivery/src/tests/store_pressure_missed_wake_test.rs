use super::support::{pressure::fill_and_refuse, temp_directory};
use crate::manager::pressure::capacity_changed;
use core::time::Duration;
use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::timeout;

struct PlentyOfSpace;

impl FreeSpace for PlentyOfSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        Some(1_000)
    }
}

#[tokio::test]
async fn capacity_change_before_wait_subscription_is_not_lost() {
    let (store, root) = bounded_store();
    let refusal = fill_and_refuse(&store).await;
    store
        .set_storage_budget(16)
        .await
        .expect("valid test fixture");
    let mut changes = store.capacity_changes();

    let changed = timeout(
        Duration::from_millis(100),
        capacity_changed(
            &store,
            &mut changes,
            Duration::from_secs(60),
            refusal.capacity_revision(),
        ),
    )
    .await
    .expect("capacity change predating subscription must remain visible");

    assert!(changed);
    std::fs::remove_dir_all(root).expect("valid test fixture");
}

fn bounded_store() -> (PartialRangeStore, std::path::PathBuf) {
    let root = temp_directory("ghostr-pressure-missed-wake");
    let limits = Limits {
        budget: 8,
        reserve: 0,
    };
    let capacity = StoreCapacity::new(limits, Arc::new(PlentyOfSpace), Duration::from_secs(60));
    let store = PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity);
    (store, root)
}
