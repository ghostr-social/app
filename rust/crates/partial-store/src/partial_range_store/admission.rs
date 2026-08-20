//! Admission control. Every byte enters the store through here, so the
//! device's free space is re-measured on the write path rather than
//! only at startup, and a cap that moved below what the store holds
//! evicts instead of merely refusing.

use crate::partial_range_store::capacity::CapacityRevision;
use crate::partial_range_store::{eviction, Entries, PartialRangeStore};
use anyhow::{Error, Result};
use log::warn;
use std::fmt;
use std::sync::atomic::Ordering;
use tokio::sync::watch;

/// A write the store could not admit: the effective cap was reached and
/// nothing unleased was left to give back. Callers above the store read
/// it as a local condition — the device is full — never as a failure of
/// whatever they were downloading from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutOfSpace {
    /// Bytes that would still have to be freed for the write to land.
    pub short: u64,
    revision: CapacityRevision,
}

impl OutOfSpace {
    pub fn capacity_revision(&self) -> CapacityRevision {
        self.revision
    }
}

impl fmt::Display for OutOfSpace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "video store is out of space: {} bytes short",
            self.short
        )
    }
}

impl std::error::Error for OutOfSpace {}

impl PartialRangeStore {
    /// Refusal decisions taken so far. Writes that repeat a standing
    /// refusal do not add one, so this counts pressure episodes rather
    /// than refused buffers.
    pub fn refusals(&self) -> u64 {
        self.refusals.load(Ordering::Relaxed)
    }

    /// Capacity changes are separate from ordinary range writes, so a
    /// parked downloader wakes only when its refusal may have changed.
    pub fn capacity_changes(&self) -> watch::Receiver<u64> {
        self.capacity.events().subscribe()
    }

    /// One forced free-space measurement for a parked delivery episode.
    /// An unchanged answer emits no event and is not polled again.
    pub async fn recheck_capacity(&self) -> u64 {
        let _capacity = self.capacity_updates.lock().await;
        let mut entries = self.entries.lock().await;
        let used = *self.used_bytes.lock().await;
        self.capacity.recheck(&self.root, used).await;
        self.enforce_locked(&mut entries).await
    }

    /// Applies a positive user budget to subsequent admissions. The
    /// entry lock makes a shrink wait for any already-admitted write,
    /// then evicts immediately against the final accounted usage.
    pub async fn set_storage_budget(&self, budget: u64) -> Result<()> {
        let _capacity = self.capacity_updates.lock().await;
        self.capacity.set_budget(budget)?;
        let mut entries = self.entries.lock().await;
        self.enforce_locked(&mut entries).await;
        Ok(())
    }

    /// Re-measures free space and evicts, least recently used first,
    /// until the store fits under the effective cap. Returns the bytes
    /// given back. Safe to call on a timer: other apps consume the same
    /// file system, so the cap shrinks without the store doing anything.
    pub async fn enforce_capacity(&self) -> u64 {
        let _capacity = self.capacity_updates.lock().await;
        let mut entries = self.entries.lock().await;
        self.enforce_locked(&mut entries).await
    }

    async fn enforce_locked(&self, entries: &mut Entries) -> u64 {
        let short = self.shortfall(0).await;
        if short == 0 {
            return 0;
        }
        let freed = self.evict(entries, "", short).await;
        // Only what actually moved is news. This runs after every chunk,
        // so reporting a shortfall the store cannot cover would be the
        // same log storm the refusal path exists to avoid; that case is
        // reported once per decision when a write is refused.
        if freed > 0 {
            warn!("Video store gave back {freed} of {short} bytes to protect free space");
        }
        freed
    }

    /// Makes room for `wanted` more bytes of `key` before anything is
    /// written. `key` is never evicted to make room for itself, so a
    /// video in progress is refused rather than silently truncated.
    pub(crate) async fn make_room(
        &self,
        entries: &mut Entries,
        key: &str,
        wanted: u64,
    ) -> Result<()> {
        let revision = self.capacity.events().revision();
        let short = self.shortfall(wanted).await;
        if short == 0 {
            return Ok(());
        }
        let freed = self.evict(entries, key, short).await;
        match freed >= short {
            true => Ok(()),
            false => Err(self.refuse(short - freed, revision).await),
        }
    }

    /// Admits transaction scratch only from existing headroom. Policy
    /// eviction must never choose an unrelated LRU victim behind the
    /// planner's back merely to construct the selected replacement.
    pub(crate) async fn require_headroom(&self, wanted: u64) -> Result<()> {
        let revision = self.capacity.events().revision();
        let short = self.shortfall(wanted).await;
        match short {
            0 => Ok(()),
            short => Err(self.refuse(short, revision).await),
        }
    }

    /// Bytes the store must give back before `wanted` more may land.
    pub(super) async fn shortfall(&self, wanted: u64) -> u64 {
        let used = *self.used_bytes.lock().await;
        let reserved = self.reserved_bytes().await;
        let cap = self.capacity.cap(&self.root, used).await;
        used.saturating_add(reserved)
            .saturating_add(wanted)
            .saturating_sub(cap)
    }

    /// One refusal decision per capacity measurement. A player pulling
    /// every buffer against a full device hits the same standing answer
    /// over and over — device pass 3 logged it sixteen times in one
    /// second — so repeats under one measurement are not counted or
    /// reported again. A new measurement, or bytes given back, is a new
    /// decision. Eviction still runs on every attempt: a lease dropped
    /// meanwhile may have made room, and refusing then would cost the
    /// user a video the store could have served.
    async fn refuse(&self, short: u64, revision: CapacityRevision) -> Error {
        let mut refused = self.refused.lock().await;
        let generation = self.capacity.generation();
        if *refused != Some(generation) {
            *refused = Some(generation);
            self.refusals.fetch_add(1, Ordering::Relaxed);
        }
        OutOfSpace { short, revision }.into()
    }

    /// Discards least recently used videos until `wanted` bytes are
    /// back. Freeing bytes raises free space by the same amount, which
    /// the capacity model is told about as the files go, so the caller
    /// can compare the result against the shortfall directly.
    async fn evict(&self, entries: &mut Entries, protected: &str, wanted: u64) -> u64 {
        let reserved = self.reserved_keys().await;
        let leased = |key: &str| self.leases.held(key) || reserved.contains(key);
        let mut staged = self.staged_response_bytes().await;
        for (key, bytes) in self.cleanup_debt_bytes().await {
            *staged.entry(key).or_default() += bytes;
        }
        let victims = eviction::victims(entries, &staged, wanted, protected, &leased);
        let mut freed = 0_u64;
        for key in victims {
            let bytes = entries
                .get(&key)
                .map_or(0, |entry| entry.accounted)
                .saturating_add(staged.get(&key).copied().unwrap_or_default());
            match self.discard(entries, &key).await {
                Ok(()) => freed = freed.saturating_add(bytes),
                Err(error) => warn!("Video store could not evict {key}: {error:#}"),
            }
        }
        freed
    }
}
