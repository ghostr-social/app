use crate::partial_range_disk as disk;
use crate::partial_range_disk::Entry;
use crate::partial_range_paths::{validate_key, StorePaths};
use crate::partial_range_store::capacity::StoreCapacity;
use crate::partial_range_store::leases::{StoreLease, StoreLeases};
use anyhow::{Context, Result};
use ghostr_engine::representation::{RepresentationBinding, SourceGeneration, TransferIdentity};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{Mutex, Notify, RwLock};

mod action;
mod admission;
pub mod capacity;
mod cleanup_debt;
mod clear;
mod discard;
mod eviction;
mod finalize;
pub mod free_space;
mod generation;
mod keyed_updates;
pub mod leases;
mod policy_eviction;
mod policy_intent;
mod queries;
mod reload;
mod replacement_cleanup;
mod representation;
mod response;
mod single_response;
mod sparse_intent;
mod transform;
mod writes;

pub use action::{ActionReservationExtension, StoreAction};
pub use admission::OutOfSpace;
pub use policy_eviction::EvictionOutcome;
pub use queries::{StoredEvidenceId, StoredMediaSnapshot};
pub use representation::{ContentRevision, RepresentationRead};
pub use response::ResponseOpenResult;
pub use transform::{TransformFence, TransformPublication, TransformPublicationOutcome};

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
    representation_updates: RwLock<()>,
    keyed_updates: keyed_updates::KeyedUpdates,
    capacity_updates: Mutex<()>,
    selected_transfers: StdMutex<HashMap<String, TransferIdentity>>,
    source_generations: Mutex<HashMap<String, (String, SourceGeneration)>>,
    sparse_response_actions: Mutex<HashMap<u64, generation::SparseResponseState>>,
    single_response_actions: Mutex<HashMap<String, single_response::SingleResponseState>>,
    action_reservations: Mutex<action::ActionReservations>,
    cleanup_debts: Mutex<cleanup_debt::CleanupDebts>,
    content_revisions: Mutex<HashMap<String, u64>>,
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
            representation_updates: RwLock::new(()),
            keyed_updates: keyed_updates::KeyedUpdates::default(),
            capacity_updates: Mutex::new(()),
            selected_transfers: StdMutex::new(HashMap::new()),
            source_generations: Mutex::new(HashMap::new()),
            sparse_response_actions: Mutex::new(HashMap::new()),
            single_response_actions: Mutex::new(HashMap::new()),
            action_reservations: Mutex::new(HashMap::new()),
            cleanup_debts: Mutex::new(HashMap::new()),
            content_revisions: Mutex::new(HashMap::new()),
        }
    }

    /// Woken (`notify_waiters`) after stored-byte or binding-authority
    /// changes; readers register before re-checking the store.
    pub fn change_notifier(&self) -> Arc<Notify> {
        self.changed.clone()
    }

    /// Pins `key` until the returned lease drops: capacity pressure
    /// evicts some other video instead of one that is in use.
    pub fn lease(&self, key: &str) -> StoreLease {
        self.leases.acquire(key)
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
