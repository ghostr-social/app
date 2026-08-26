use crate::partial_range_store::capacity::{Limits, StoreCapacity};
use crate::partial_range_store::free_space::FreeSpace;
use crate::partial_range_store::PartialRangeStore;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(in crate::tests) struct FakeSpace {
    available: AtomicU64,
}

impl FakeSpace {
    pub(in crate::tests) fn new(available: u64) -> Arc<Self> {
        Arc::new(Self {
            available: AtomicU64::new(available),
        })
    }

    pub(in crate::tests) fn set(&self, available: u64) {
        self.available.store(available, Ordering::SeqCst);
    }
}

impl FreeSpace for FakeSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        Some(self.available.load(Ordering::SeqCst))
    }
}

pub(in crate::tests) struct SpacedStore {
    pub(in crate::tests) store: PartialRangeStore,
    pub(in crate::tests) used_bytes: Arc<Mutex<u64>>,
    pub(in crate::tests) space: Arc<FakeSpace>,
    pub(in crate::tests) root: PathBuf,
    limits: Limits,
    recheck: Duration,
}

pub(in crate::tests) fn spaced_store(prefix: &str, limits: Limits, available: u64) -> SpacedStore {
    paced_store(prefix, limits, available, Duration::ZERO)
}

pub(in crate::tests) fn paced_store(
    prefix: &str,
    limits: Limits,
    available: u64,
    recheck: Duration,
) -> SpacedStore {
    on_disk(
        super::temp_root(prefix),
        FakeSpace::new(available),
        limits,
        recheck,
    )
}

pub(in crate::tests) fn reopened(fixture: &SpacedStore) -> SpacedStore {
    on_disk(
        fixture.root.clone(),
        std::sync::Arc::clone(&fixture.space),
        fixture.limits,
        fixture.recheck,
    )
}

fn on_disk(root: PathBuf, space: Arc<FakeSpace>, limits: Limits, recheck: Duration) -> SpacedStore {
    let used_bytes = Arc::new(Mutex::new(0));
    let capacity = StoreCapacity::new(limits, std::sync::Arc::<FakeSpace>::clone(&space), recheck);
    SpacedStore {
        store: PartialRangeStore::with_capacity(
            root.clone(),
            std::sync::Arc::clone(&used_bytes),
            capacity,
        ),
        used_bytes,
        space,
        root,
        limits,
        recheck,
    }
}

pub(in crate::tests) fn limits(budget: u64, reserve: u64) -> Limits {
    Limits { budget, reserve }
}

pub(in crate::tests) fn plain_store(
    root: PathBuf,
    used_bytes: Arc<Mutex<u64>>,
) -> PartialRangeStore {
    PartialRangeStore::with_capacity(root, used_bytes, StoreCapacity::system(u64::MAX))
}
