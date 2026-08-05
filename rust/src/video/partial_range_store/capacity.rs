//! What the store is allowed to occupy right now. The user's budget is
//! an upper bound, never a promise: the effective cap is whichever of
//! the budget and the device's real free space is smaller, minus a
//! reserve the store must never spend.

use crate::video::partial_range_store::free_space::{FreeSpace, SystemFreeSpace};
use anyhow::{ensure, Result};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// Free space the store leaves to the rest of the device, so caching
/// videos can never take the file system to zero.
pub const DEFAULT_RESERVE_BYTES: u64 = 256 * 1024 * 1024;

/// How long one free-space measurement is trusted. Short enough that a
/// device filling up is noticed within a chunk or two.
pub const DEFAULT_RECHECK: Duration = Duration::from_secs(2);

/// The two ceilings the store obeys.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// What the user configured; `u64::MAX` means "no budget of its own".
    pub budget: u64,
    /// Free space that must survive whatever the store does.
    pub reserve: u64,
}

impl Limits {
    /// `budget` against the default device reserve.
    pub fn budget(budget: u64) -> Self {
        Self {
            budget,
            reserve: DEFAULT_RESERVE_BYTES,
        }
    }
}

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
}

impl StoreCapacity {
    pub fn new(limits: Limits, space: Arc<dyn FreeSpace>) -> Self {
        Self {
            budget: AtomicU64::new(limits.budget),
            reserve: limits.reserve,
            space,
            recheck: DEFAULT_RECHECK,
            sample: Mutex::new(None),
            generations: AtomicU64::new(0),
        }
    }

    /// The device's own file system under `budget`.
    pub fn system(budget: u64) -> Self {
        Self::new(Limits::budget(budget), Arc::new(SystemFreeSpace))
    }

    pub fn with_recheck(mut self, recheck: Duration) -> Self {
        self.recheck = recheck;
        self
    }

    /// The most the store may occupy, given the `used` bytes it already
    /// holds: `min(budget, used + free - reserve)`, never below zero.
    pub async fn cap(&self, root: &Path, used: u64) -> u64 {
        let mut sample = self.sample.lock().await;
        cap_for(self.limits(), used, self.current(&mut sample, root))
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
    fn current(&self, sample: &mut Option<Sample>, root: &Path) -> Option<u64> {
        if let Some(fresh) = sample.as_ref().filter(|s| s.taken.elapsed() < self.recheck) {
            return fresh.available;
        }
        let available = self.space.available_bytes(root);
        *sample = Some(Sample {
            taken: Instant::now(),
            available,
            generation: self.next_generation(),
        });
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
