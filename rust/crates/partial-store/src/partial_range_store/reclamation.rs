use super::{eviction, Entries, PartialRangeStore};
use log::warn;
use std::collections::HashSet;

impl PartialRangeStore {
    /// Makes room for a current request from cache entries outside the live
    /// working set. Read leases and action reservations remain protected.
    /// Returns the number of bytes actually released.
    pub async fn reclaim_outside(&self, working_set: &HashSet<String>, wanted: u64) -> u64 {
        if wanted == 0 {
            return 0;
        }
        let mut entries = self.entries.lock().await;
        let short = self.shortfall(wanted).await;
        if short == 0 {
            return 0;
        }
        self.evict_excluding(&mut entries, short, |key| working_set.contains(key))
            .await
    }

    pub(super) async fn evict(&self, entries: &mut Entries, protected: &str, wanted: u64) -> u64 {
        self.evict_excluding(entries, wanted, |key| key == protected)
            .await
    }

    async fn evict_excluding(
        &self,
        entries: &mut Entries,
        wanted: u64,
        excluded: impl Fn(&str) -> bool + Send + Sync,
    ) -> u64 {
        let reserved = self.reserved_keys().await;
        let protected =
            |key: &str| excluded(key) || self.leases.held(key) || reserved.contains(key);
        let mut staged = self.staged_response_bytes().await;
        for (key, bytes) in self.cleanup_debt_bytes().await {
            *staged.entry(key).or_default() += bytes;
        }
        let victims = eviction::victims(entries, &staged, wanted, &protected);
        self.discard_victims(entries, &staged, victims).await
    }

    async fn discard_victims(
        &self,
        entries: &mut Entries,
        staged: &std::collections::BTreeMap<String, u64>,
        victims: Vec<String>,
    ) -> u64 {
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
