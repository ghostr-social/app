//! Pins that keep one video in the store while something is using it,
//! so capacity pressure never pulls a file out from under a reader.

use super::capacity::CapacityEvents;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Reference counts of the keys callers are actively holding.
pub struct StoreLeases {
    counts: Mutex<HashMap<String, u32>>,
    events: CapacityEvents,
}

impl StoreLeases {
    pub(super) fn new(events: CapacityEvents) -> Self {
        Self {
            counts: Mutex::new(HashMap::new()),
            events,
        }
    }

    pub(super) fn acquire(self: &Arc<Self>, key: &str) -> StoreLease {
        *self.counts().entry(key.to_owned()).or_insert(0) += 1;
        StoreLease {
            leases: std::sync::Arc::clone(self),
            key: key.to_owned(),
        }
    }

    pub(super) fn try_acquire_unheld(self: &Arc<Self>, key: &str) -> Option<StoreLease> {
        let mut counts = self.counts();
        if counts.contains_key(key) {
            return None;
        }
        counts.insert(key.to_owned(), 1);
        Some(StoreLease {
            leases: std::sync::Arc::clone(self),
            key: key.to_owned(),
        })
    }

    /// True while at least one lease on `key` is alive.
    pub(super) fn held(&self, key: &str) -> bool {
        self.counts().get(key).is_some_and(|count| *count > 0)
    }

    fn release(&self, key: &str) {
        let mut counts = self.counts();
        let Some(count) = counts.get_mut(key) else {
            return;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            counts.remove(key);
            self.events.signal();
        }
    }

    /// A poisoned lease table still describes which keys are in use, so
    /// the counts are read through the poison rather than panicking.
    fn counts(&self) -> MutexGuard<'_, HashMap<String, u32>> {
        self.counts.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A hold on one stored video. While it lives, eviction picks another
/// video instead; dropping it releases the hold.
pub struct StoreLease {
    leases: Arc<StoreLeases>,
    key: String,
}

impl StoreLease {
    pub(super) fn is_exclusive(&self) -> bool {
        self.leases
            .counts()
            .get(&self.key)
            .is_some_and(|count| *count == 1)
    }
}

impl Drop for StoreLease {
    fn drop(&mut self) {
        self.leases.release(&self.key);
    }
}
