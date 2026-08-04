use crate::video::partial_range_disk as disk;
use crate::video::partial_range_disk::Entry;
use crate::video::partial_range_manifest::RangeManifest;
use crate::video::partial_range_paths::{validate_key, StorePaths};
use crate::video::partial_range_store::capacity::StoreCapacity;
use crate::video::partial_range_store::leases::{StoreLease, StoreLeases};
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

mod admission;
pub mod capacity;
mod eviction;
mod finalize;
pub mod free_space;
pub mod leases;
mod queries;

pub(crate) type Entries = HashMap<String, Entry>;

/// Sparse on-disk store of partially downloaded videos: per key one data
/// file written at byte offsets plus a persisted manifest of the present
/// ranges. What it may occupy is not the configured budget alone but
/// [`PartialRangeStore::effective_capacity`] — the budget capped by the
/// device's real free space.
pub struct PartialRangeStore {
    root: PathBuf,
    paths: StorePaths,
    entries: Mutex<Entries>,
    used_bytes: Arc<Mutex<u64>>,
    changed: Arc<Notify>,
    capacity: StoreCapacity,
    leases: Arc<StoreLeases>,
    clock: AtomicU64,
}

/// The manifest a write would produce and the bytes it would add.
struct PlannedWrite {
    manifest: RangeManifest,
    added: u64,
}

impl PartialRangeStore {
    /// A store bounded by the device's free space alone. Callers that
    /// know the user's budget use [`Self::with_capacity`].
    pub fn new(root: PathBuf, used_bytes: Arc<Mutex<u64>>) -> Self {
        Self::with_capacity(root, used_bytes, StoreCapacity::system(u64::MAX))
    }

    pub fn with_capacity(
        root: PathBuf,
        used_bytes: Arc<Mutex<u64>>,
        capacity: StoreCapacity,
    ) -> Self {
        Self {
            root: root.clone(),
            paths: StorePaths::new(root),
            entries: Mutex::new(HashMap::new()),
            used_bytes,
            changed: Arc::new(Notify::new()),
            capacity,
            leases: Arc::default(),
            clock: AtomicU64::new(0),
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

    pub async fn write_range(&self, key: &str, offset: u64, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let end = offset
            .checked_add(bytes.len() as u64)
            .context("partial range end overflows")?;
        let mut entries = self.entries.lock().await;
        let plan = self.plan_write(&mut entries, key, offset..end).await?;
        self.make_room(&mut entries, key, plan.added).await?;
        disk::write_at(&self.paths.partial(key), offset, bytes).await?;
        disk::save_manifest(&self.paths.manifest(key), &plan.manifest).await?;
        self.record_write(&mut entries, key, plan).await
    }

    pub async fn set_total_len(&self, key: &str, len: u64) -> Result<()> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.completion.is_some() {
            bail!("cannot resize a finalized video");
        }
        entry.manifest.set_total_len(len)?;
        disk::save_manifest(&self.paths.manifest(key), &entry.manifest).await?;
        self.changed.notify_waiters();
        Ok(())
    }

    pub async fn remove(&self, key: &str) -> Result<()> {
        let mut entries = self.entries.lock().await;
        self.entry(&mut entries, key).await?;
        self.discard(&mut entries, key).await
    }

    pub fn completed_path(&self, key: &str) -> PathBuf {
        self.paths.completed(key)
    }

    /// What the write would leave behind, refusing a finalized key. The
    /// manifest is only a plan here: nothing touches disk until the
    /// bytes are admitted.
    async fn plan_write(
        &self,
        entries: &mut Entries,
        key: &str,
        span: Range<u64>,
    ) -> Result<PlannedWrite> {
        let entry = self.entry(entries, key).await?;
        if entry.completion.is_some() {
            bail!("cannot write into a finalized video");
        }
        let mut manifest = entry.manifest.clone();
        manifest.insert(span)?;
        let added = manifest.covered_bytes() - entry.manifest.covered_bytes();
        Ok(PlannedWrite { manifest, added })
    }

    /// Bookkeeping for bytes that are already on disk.
    async fn record_write(
        &self,
        entries: &mut Entries,
        key: &str,
        plan: PlannedWrite,
    ) -> Result<()> {
        let touched = self.tick();
        let entry = self.entry(entries, key).await?;
        entry.manifest = plan.manifest;
        entry.accounted += plan.added;
        entry.touched = touched;
        self.credit(plan.added).await;
        self.changed.notify_waiters();
        Ok(())
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

    pub(crate) async fn entry<'a>(
        &self,
        entries: &'a mut Entries,
        key: &str,
    ) -> Result<&'a mut Entry> {
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

    async fn release(&self, bytes: u64) {
        let mut used = self.used_bytes.lock().await;
        *used = used.saturating_sub(bytes);
    }
}
