//! Admission control. Every byte enters the store through here, so the
//! device's free space is re-measured on the write path rather than
//! only at startup, and a cap that moved below what the store holds
//! evicts instead of merely refusing.

use crate::video::partial_range_store::{eviction, Entries, PartialRangeStore};
use anyhow::{bail, Result};
use log::warn;

impl PartialRangeStore {
    /// The most this store may occupy right now: the configured budget,
    /// capped by what the file system can spare above its reserve.
    pub async fn effective_capacity(&self) -> u64 {
        let used = *self.used_bytes.lock().await;
        self.capacity.cap(&self.root, used).await
    }

    /// Re-measures free space and evicts, least recently used first,
    /// until the store fits under the effective cap. Returns the bytes
    /// given back. Safe to call on a timer: other apps consume the same
    /// file system, so the cap shrinks without the store doing anything.
    pub async fn enforce_capacity(&self) -> u64 {
        let short = self.shortfall(0).await;
        if short == 0 {
            return 0;
        }
        let mut entries = self.entries.lock().await;
        let freed = self.evict(&mut entries, "", short).await;
        warn!("Video store gave back {freed} of {short} bytes to protect free space");
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
        let short = self.shortfall(wanted).await;
        if short == 0 {
            return Ok(());
        }
        let freed = self.evict(entries, key, short).await;
        if freed >= short {
            return Ok(());
        }
        bail!("video store is out of space: {} bytes short for {key}", short - freed)
    }

    /// Bytes the store must give back before `wanted` more may land.
    async fn shortfall(&self, wanted: u64) -> u64 {
        let used = *self.used_bytes.lock().await;
        let cap = self.capacity.cap(&self.root, used).await;
        used.saturating_add(wanted).saturating_sub(cap)
    }

    /// Discards least recently used videos until `wanted` bytes are
    /// back. Freeing bytes raises free space by the same amount, which
    /// is why the caller can compare the result against the shortfall
    /// without measuring the file system again.
    async fn evict(&self, entries: &mut Entries, protected: &str, wanted: u64) -> u64 {
        let leased = |key: &str| self.leases.held(key);
        let victims = eviction::victims(entries, wanted, protected, &leased);
        let mut freed = 0_u64;
        for key in victims {
            let bytes = entries.get(&key).map_or(0, |entry| entry.accounted);
            match self.discard(entries, &key).await {
                Ok(()) => freed = freed.saturating_add(bytes),
                Err(error) => warn!("Video store could not evict {key}: {error:#}"),
            }
        }
        freed
    }
}
