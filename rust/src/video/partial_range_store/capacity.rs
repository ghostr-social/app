//! What the store is allowed to occupy right now. The user's budget is
//! an upper bound, never a promise: the effective cap is whichever of
//! the budget and the device's real free space is smaller, minus a
//! reserve the store must never spend.

use crate::video::partial_range_store::free_space::{FreeSpace, SystemFreeSpace};
use std::path::Path;
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
}

/// Measures free space (at most once per recheck window) and turns the
/// measurement into the store's effective cap.
pub struct StoreCapacity {
    limits: Limits,
    space: Arc<dyn FreeSpace>,
    recheck: Duration,
    sample: Mutex<Option<Sample>>,
}

impl StoreCapacity {
    pub fn new(limits: Limits, space: Arc<dyn FreeSpace>) -> Self {
        Self {
            limits,
            space,
            recheck: DEFAULT_RECHECK,
            sample: Mutex::new(None),
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
        cap_for(self.limits, used, self.available(root).await)
    }

    async fn available(&self, root: &Path) -> Option<u64> {
        let mut sample = self.sample.lock().await;
        if let Some(current) = sample.as_ref().filter(|s| s.taken.elapsed() < self.recheck) {
            return current.available;
        }
        let available = self.space.available_bytes(root);
        *sample = Some(Sample {
            taken: Instant::now(),
            available,
        });
        available
    }
}

/// `used` belongs in the sum because bytes the store already holds are
/// bytes the file system already reports as taken: giving them back
/// raises free space by exactly as much.
fn cap_for(limits: Limits, used: u64, available: Option<u64>) -> u64 {
    let Some(available) = available else {
        return limits.budget;
    };
    let spendable =
        i128::from(used) + i128::from(available) - i128::from(limits.reserve);
    u64::try_from(spendable.clamp(0, i128::from(limits.budget))).unwrap_or(0)
}
