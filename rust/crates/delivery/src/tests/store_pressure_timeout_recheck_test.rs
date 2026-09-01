use super::support::{pressure::fill_and_refuse, temp_directory};
use crate::manager::pressure::capacity_changed;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::timeout;

struct MutableSpace(AtomicU64);

impl MutableSpace {
    fn set(&self, bytes: u64) {
        self.0.store(bytes, Ordering::SeqCst);
    }
}

impl FreeSpace for MutableSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        Some(self.0.load(Ordering::SeqCst))
    }
}

#[tokio::test]
async fn pressure_timeout_rechecks_external_capacity_before_resuming() {
    let (store, space, root) = recheck_store();
    let refusal = fill_and_refuse(&store).await;
    let mut changes = store.capacity_changes();
    space.set(16);

    let changed = timeout(
        Duration::from_millis(100),
        capacity_changed(
            &store,
            &mut changes,
            Duration::ZERO,
            refusal.capacity_revision(),
        ),
    )
    .await
    .expect("the pressure timeout must complete");

    assert!(changed, "new external capacity must wake delivery");
    std::fs::remove_dir_all(root).expect("valid test fixture");
}

fn recheck_store() -> (PartialRangeStore, Arc<MutableSpace>, PathBuf) {
    let root = temp_directory("ghostr-pressure-timeout-recheck");
    let space = Arc::new(MutableSpace(AtomicU64::new(8)));
    let capacity = StoreCapacity::new(
        Limits {
            budget: 16,
            reserve: 0,
        },
        std::sync::Arc::<MutableSpace>::clone(&space),
        Duration::from_secs(60),
    );
    let store = PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity);
    (store, space, root)
}
