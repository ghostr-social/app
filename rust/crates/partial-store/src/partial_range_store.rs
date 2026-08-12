use crate::partial_range_disk as disk;
use crate::partial_range_disk::Entry;
use crate::partial_range_paths::{validate_key, StorePaths};
use crate::partial_range_store::capacity::StoreCapacity;
use crate::partial_range_store::leases::{StoreLease, StoreLeases};
use anyhow::{Context, Result};
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{Mutex, Notify};

mod admission;
pub mod capacity;
mod clear;
mod eviction;
mod finalize;
pub mod free_space;
pub mod leases;
mod policy_eviction;
mod queries;
mod reload;
mod representation;
mod writes;

pub use admission::OutOfSpace;
pub use representation::RepresentationRead;

pub(crate) type Entries = HashMap<String, Entry>;

/// Sparse on-disk store of partially downloaded videos: per key one data
/// file written at byte offsets plus a persisted manifest of the present
/// ranges. Its configured budget is capped by the device's real free
/// space.
pub struct PartialRangeStore {
    root: PathBuf,
    paths: StorePaths,
    entries: Mutex<Entries>,
    used_bytes: Arc<Mutex<u64>>,
    changed: Arc<Notify>,
    capacity: StoreCapacity,
    leases: Arc<StoreLeases>,
    clock: AtomicU64,
    /// The capacity measurement a refusal was last decided against.
    refused: Mutex<Option<u64>>,
    refusals: AtomicU64,
    representations: Mutex<HashMap<String, RepresentationBinding>>,
    representation_updates: Mutex<()>,
    selected_transfers: StdMutex<HashMap<String, TransferIdentity>>,
}

impl PartialRangeStore {
    pub fn with_capacity(
        root: PathBuf,
        used_bytes: Arc<Mutex<u64>>,
        capacity: StoreCapacity,
    ) -> Self {
        let leases = Arc::new(StoreLeases::new(capacity.events()));
        Self {
            root: root.clone(),
            paths: StorePaths::new(root),
            entries: Mutex::new(HashMap::new()),
            used_bytes,
            changed: Arc::new(Notify::new()),
            capacity,
            leases,
            clock: AtomicU64::new(0),
            refused: Mutex::new(None),
            refusals: AtomicU64::new(0),
            representations: Mutex::new(HashMap::new()),
            representation_updates: Mutex::new(()),
            selected_transfers: StdMutex::new(HashMap::new()),
        }
    }

    /// Woken (`notify_waiters`) after every range write, every total
    /// length declaration and every promotion out of the partial pool;
    /// readers register before re-checking the store.
    pub fn change_notifier(&self) -> Arc<Notify> {
        self.changed.clone()
    }

    /// Pins `key` until the returned lease drops: capacity pressure
    /// evicts some other video instead of one that is in use.
    pub fn lease(&self, key: &str) -> StoreLease {
        self.leases.acquire(key)
    }

    async fn discard(&self, entries: &mut Entries, key: &str) -> Result<()> {
        for path in self.paths.all(key) {
            disk::remove_if_present(&path).await?;
        }
        if let Some(entry) = entries.remove(key) {
            self.release(entry.accounted).await;
        }
        Ok(())
    }

    async fn entry<'a>(&self, entries: &'a mut Entries, key: &str) -> Result<&'a mut Entry> {
        validate_key(key)?;
        if !entries.contains_key(key) {
            let loaded = disk::load_entry(&self.paths, key).await?;
            self.credit(loaded.accounted).await;
            entries.insert(key.to_owned(), loaded);
        }
        entries.get_mut(key).context("partial entry present")
    }

    /// Monotonic use counter: newer means more recently used.
    fn tick(&self) -> u64 {
        self.clock.fetch_add(1, Ordering::Relaxed).saturating_add(1)
    }

    async fn credit(&self, bytes: u64) {
        let mut used = self.used_bytes.lock().await;
        *used = used.saturating_add(bytes);
    }

    /// Bytes leaving the store for good. The file system really does get
    /// them back, so the capacity model is told before the next write
    /// asks whether there is room.
    async fn release(&self, bytes: u64) {
        let mut used = self.used_bytes.lock().await;
        *used = used.saturating_sub(bytes);
        drop(used);
        self.capacity.gave_back(bytes).await;
    }
}
