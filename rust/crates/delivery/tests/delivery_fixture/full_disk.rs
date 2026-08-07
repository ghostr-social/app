//! A store backed by a filesystem that reports whatever free space the
//! test names, so delivery can meet a full device without filling one.

use super::temp_directory;
use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

struct FixedSpace {
    available: u64,
}

impl FreeSpace for FixedSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        Some(self.available)
    }
}

pub struct SpacedStore {
    pub store: PartialRangeStore,
    pub root: PathBuf,
}

/// Re-measures free space on every check, so the cap the store enforces
/// is always the one this fixture was built with.
pub fn spaced_store(prefix: &str, limits: Limits, available: u64) -> SpacedStore {
    let root = temp_directory(prefix);
    let capacity =
        StoreCapacity::new(limits, Arc::new(FixedSpace { available })).with_recheck(Duration::ZERO);
    SpacedStore {
        store: PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity),
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
