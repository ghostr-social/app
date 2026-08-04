#![allow(dead_code)]

//! A filesystem whose free space the test moves at will, so the store's
//! effective cap can be pushed around without filling a real disk.

use rust_lib_ghostr::video::partial_range_store::capacity::{Limits, StoreCapacity};
use rust_lib_ghostr::video::partial_range_store::free_space::FreeSpace;
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub struct FakeSpace {
    available: AtomicU64,
}

impl FakeSpace {
    pub fn new(available: u64) -> Arc<Self> {
        Arc::new(Self {
            available: AtomicU64::new(available),
        })
    }

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
    pub used_bytes: Arc<Mutex<u64>>,
    pub space: Arc<FakeSpace>,
    pub root: PathBuf,
}

/// A store that re-measures free space on every check, so a test sees
/// its move immediately.
pub fn spaced_store(prefix: &str, limits: Limits, available: u64) -> SpacedStore {
    let root = temp_root(prefix);
    let space = FakeSpace::new(available);
    let used_bytes = Arc::new(Mutex::new(0));
    let capacity = StoreCapacity::new(limits, space.clone()).with_recheck(Duration::ZERO);
    SpacedStore {
        store: PartialRangeStore::with_capacity(root.clone(), used_bytes.clone(), capacity),
        used_bytes,
        space,
        root,
    }
}

pub fn limits(budget: u64, reserve: u64) -> Limits {
    Limits { budget, reserve }
}

pub fn temp_root(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

pub fn discard(root: &Path) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("remove store");
    }
}
