//! What the store is allowed to occupy right now. The user's budget is
//! an upper bound, never a promise: the effective cap is whichever of
//! the budget and the device's real free space is smaller, minus a
//! reserve the store must never spend.

use crate::partial_range_store::free_space::{FreeSpace, SystemFreeSpace};
use anyhow::{ensure, Result};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

mod events;
mod limits;
pub(crate) use events::CapacityEvents;
pub use events::CapacityRevision;
pub use limits::{CapacitySnapshot, Limits};

/// Free space the store leaves to the rest of the device, so caching
/// videos can never take the file system to zero.
const DEFAULT_RESERVE_BYTES: u64 = 256 * 1024 * 1024;

/// How long one free-space measurement is trusted. Short enough that a
/// device filling up is noticed within a chunk or two.
pub const DEFAULT_RECHECK: Duration = Duration::from_secs(2);

struct Sample {
    taken: Instant,
    available: Option<u64>,
    generation: u64,
}

/// Measures free space (at most once per recheck window) and turns the
/// measurement into the store's effective cap. Between measurements the
/// store's own writes and evictions move the standing measurement, so a
/// cap taken inside one window still describes the file system.
pub struct StoreCapacity {
    budget: AtomicU64,
    reserve: u64,
    space: Arc<dyn FreeSpace>,
    recheck: Duration,
    sample: Mutex<Option<Sample>>,
    generations: AtomicU64,
    events: CapacityEvents,
}

impl StoreCapacity {
    pub fn new(limits: Limits, space: Arc<dyn FreeSpace>, recheck: Duration) -> Self {
        Self {
            budget: AtomicU64::new(limits.budget),
            reserve: limits.reserve,
            space,
            recheck,
            sample: Mutex::new(None),
            generations: AtomicU64::new(0),
            events: CapacityEvents::new(),
        }
    }

    /// The device's own file system under `budget`.
    pub fn system(budget: u64) -> Self {
        Self::new(
            Limits::budget(budget),
            Arc::new(SystemFreeSpace),
            DEFAULT_RECHECK,
        )
    }

    /// The most the store may occupy, given the `used` bytes it already
    /// holds: `min(budget, used + free - reserve)`, never below zero.
    pub async fn cap(&self, root: &Path, used: u64) -> u64 {
        let mut sample = self.sample.lock().await;
        cap_for(self.limits(), used, self.current(&mut sample, root, used))
    }

    pub(crate) async fn recheck(&self, root: &Path, used: u64) {
        let mut sample = self.sample.lock().await;
        self.measure(&mut sample, root, used);
    }

    pub(crate) fn events(&self) -> CapacityEvents {
        self.events.clone()
    }

    /// Replaces the user ceiling for future decisions. A real change
    /// invalidates a standing refusal even when free space is unchanged.
    pub fn set_budget(&self, budget: u64) -> Result<bool> {
        ensure!(budget > 0, "video store budget must be positive");
        let previous = self.budget.swap(budget, Ordering::SeqCst);
        if previous == budget {
            return Ok(false);
        }
        self.next_generation();
        self.events.signal();
        Ok(true)
    }

    /// Identifies the measurement in force right now: it changes when
    /// free space is re-measured and when the store gives bytes back,
    /// which are the only two things that can turn a refusal around.
    pub fn generation(&self) -> u64 {
        self.generations.load(Ordering::SeqCst)
    }

    /// The store handed `bytes` back to the file system, so the standing
    /// measurement understates free space by exactly that much. Without
    /// this an eviction is invisible until the next syscall, and the
    /// store goes on refusing writes it has just made room for.
    pub async fn gave_back(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        let mut sample = self.sample.lock().await;
        let Some(current) = sample.as_mut() else {
            return;
        };
        current.available = current.available.map(|free| free.saturating_add(bytes));
        current.generation = self.next_generation();
        self.events.signal();
    }

    pub(crate) fn released_reservation(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        self.next_generation();
        self.events.signal();
    }

    /// The store spent `bytes`: the same measurement, that much less of
    /// it left. Keeps the reserve exact between measurements.
    pub async fn spent(&self, bytes: u64) {
        let mut sample = self.sample.lock().await;
        let Some(current) = sample.as_mut() else {
            return;
        };
        current.available = current.available.map(|free| free.saturating_sub(bytes));
    }

    /// The standing measurement, re-measuring once the window is up.
    fn current(&self, sample: &mut Option<Sample>, root: &Path, used: u64) -> Option<u64> {
        if let Some(fresh) = sample.as_ref().filter(|s| s.taken.elapsed() < self.recheck) {
            return fresh.available;
        }
        self.measure(sample, root, used)
    }

    fn measure(&self, sample: &mut Option<Sample>, root: &Path, used: u64) -> Option<u64> {
        let available = self.space.available_bytes(root);
        let limits = self.limits();
        let changed = sample.as_ref().is_some_and(|previous| {
            cap_for(limits, used, previous.available) != cap_for(limits, used, available)
        });
        *sample = Some(Sample {
            taken: Instant::now(),
            available,
            generation: self.next_generation(),
        });
        if changed {
            self.events.signal();
        }
        available
    }

    fn next_generation(&self) -> u64 {
        self.generations
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1)
    }

    fn limits(&self) -> Limits {
        Limits {
            budget: self.budget.load(Ordering::SeqCst),
            reserve: self.reserve,
        }
    }
}

/// `used` belongs in the sum because bytes the store already holds are
/// bytes the file system already reports as taken: giving them back
/// raises free space by exactly as much.
fn cap_for(limits: Limits, used: u64, available: Option<u64>) -> u64 {
    let Some(available) = available else {
        return limits.budget;
    };
    let spendable = i128::from(used) + i128::from(available) - i128::from(limits.reserve);
    u64::try_from(spendable.clamp(0, i128::from(limits.budget))).unwrap_or(0)
}
