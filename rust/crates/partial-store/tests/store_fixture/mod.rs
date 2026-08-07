#![allow(dead_code)]
//! Roots the store may be built over: a plain temp directory, or a
//! filesystem whose free space the test moves at will, so the store's
//! effective cap can be pushed around without filling a real disk.

use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
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
    let capacity = StoreCapacity::new(limits, space.clone()).with_recheck(recheck);
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

/// A directory no other caller holds. The clock alone cannot promise
/// that: it repeats a nanosecond reading often enough that two fixtures
/// built in the same instant would share a root, so the process and a
/// per-call counter carry the uniqueness and the reading only separates
/// this run from an earlier one that left a directory behind.
pub fn temp_root(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{nonce}-{process}-{sequence}"))
}

pub fn discard(root: &Path) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("remove store");
    }
}
