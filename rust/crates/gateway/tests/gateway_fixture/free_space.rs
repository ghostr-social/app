//! A store whose backing filesystem reports free space the test can move
//! mid-request, so a lease can be checked against a device that fills up
//! while a response is still streaming.

use super::temp_directory;
use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct FakeSpace {
    available: AtomicU64,
}

impl FakeSpace {
    pub fn set(&self, available: u64) {
        self.available.store(available, Ordering::SeqCst);
    }
}

impl FreeSpace for FakeSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        Some(self.available.load(Ordering::SeqCst))
    }
}

pub struct SpacedStore {
    pub store: PartialRangeStore,
    pub space: Arc<FakeSpace>,
    pub root: PathBuf,
}

/// Re-measures free space on every check, so a test sees its move
/// immediately.
pub fn spaced_store(prefix: &str, limits: Limits, available: u64) -> SpacedStore {
    let root = temp_directory(prefix);
    let space = Arc::new(FakeSpace {
        available: AtomicU64::new(available),
    });
    let capacity = StoreCapacity::new(limits, space.clone(), Duration::ZERO);
    SpacedStore {
        store: PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity),
        space,
        root,
    }
}

pub fn limits(budget: u64, reserve: u64) -> Limits {
    Limits { budget, reserve }
}

pub fn discard(root: &Path) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("remove store");
    }
}
