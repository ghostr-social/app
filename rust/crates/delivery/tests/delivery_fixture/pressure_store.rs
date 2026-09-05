use super::temp_directory;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MutableSpace(AtomicU64);

impl MutableSpace {
    pub fn set(&self, bytes: u64) {
        self.0.store(bytes, Ordering::SeqCst);
    }
}

impl FreeSpace for MutableSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        Some(self.0.load(Ordering::SeqCst))
    }
}

pub fn movable_store(prefix: &str) -> (Arc<PartialRangeStore>, Arc<MutableSpace>, PathBuf) {
    let root = temp_directory(prefix);
    let space = Arc::new(MutableSpace(AtomicU64::new(16)));
    let capacity = StoreCapacity::new(
        Limits {
            budget: 32,
            reserve: 0,
        },
        Arc::<MutableSpace>::clone(&space),
        Duration::ZERO,
    );
    let store = PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity);
    (Arc::new(store), space, root)
}
