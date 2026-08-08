#![allow(dead_code)]
//! Test stores use either a temp root or controllable free space.

use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

mod paths;

pub fn discard(root: &Path) {
    paths::discard(root);
}

pub fn temp_root(prefix: &str) -> PathBuf {
    paths::temp_root(prefix)
}

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
    pub limits: Limits,
    pub recheck: Duration,
}

/// A store that re-measures free space on every check, so a test sees
/// its move immediately.
pub fn spaced_store(prefix: &str, limits: Limits, available: u64) -> SpacedStore {
    paced_store(prefix, limits, available, Duration::ZERO)
}

/// A store that holds one free-space measurement for `recheck`.
pub fn paced_store(prefix: &str, limits: Limits, available: u64, recheck: Duration) -> SpacedStore {
    on_disk(
        temp_root(prefix),
        FakeSpace::new(available),
        limits,
        recheck,
    )
}

/// Reopens the same root with fresh in-memory state.
pub fn reopened(fixture: &SpacedStore) -> SpacedStore {
    on_disk(
        fixture.root.clone(),
        fixture.space.clone(),
        fixture.limits,
        fixture.recheck,
    )
}

fn on_disk(root: PathBuf, space: Arc<FakeSpace>, limits: Limits, recheck: Duration) -> SpacedStore {
    let used_bytes = Arc::new(Mutex::new(0));
    let capacity = StoreCapacity::new(limits, space.clone(), recheck);
    SpacedStore {
        store: PartialRangeStore::with_capacity(root.clone(), used_bytes.clone(), capacity),
        used_bytes,
        space,
        root,
        limits,
        recheck,
    }
}

pub fn limits(budget: u64, reserve: u64) -> Limits {
    Limits { budget, reserve }
}

pub fn plain_store(root: PathBuf, used_bytes: Arc<Mutex<u64>>) -> PartialRangeStore {
    PartialRangeStore::with_capacity(root, used_bytes, StoreCapacity::system(u64::MAX))
}
