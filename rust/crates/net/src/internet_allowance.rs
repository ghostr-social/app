//! Cumulative Internet admission, independent of refillable rate buckets.
//!
//! Reservations are durable before IO. An abandoned response retains its entire
//! authorized envelope because unread transport buffers have no release proof.

use anyhow::{ensure, Context as _, Result};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

mod denied;
mod disk;
mod reservation;
pub use denied::InternetAdmissionDenied;
pub(crate) use reservation::InternetReservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InternetDataLimit {
    Unlimited,
    Bytes(u64),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct InternetUsage {
    /// Includes the conservative envelope of abandoned or crash-interrupted IO.
    charged_bytes: u64,
    reserved_bytes: u64,
}

#[derive(Clone)]
pub struct InternetAllowance {
    inner: Arc<Mutex<State>>,
}

struct State {
    limit: InternetDataLimit,
    usage: InternetUsage,
    disk: Option<disk::LedgerDisk>,
    failed: bool,
}

impl InternetAllowance {
    /// Opens the exclusive durable ledger and charges orphaned reservations.
    ///
    /// # Errors
    /// Returns an error for corrupt accounting, another owner, or failed persistence.
    pub fn open(path: &Path, limit: InternetDataLimit) -> Result<Self> {
        let disk = disk::LedgerDisk::open(path)?;
        let mut usage = disk.load()?;
        usage.charged_bytes = usage
            .charged_bytes
            .checked_add(usage.reserved_bytes)
            .context("Internet usage overflow")?;
        usage.reserved_bytes = 0;
        disk.save(usage)?;
        Ok(Self::from_state(limit, usage, Some(disk)))
    }

    fn from_state(
        limit: InternetDataLimit,
        usage: InternetUsage,
        disk: Option<disk::LedgerDisk>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(State {
                limit,
                usage,
                disk,
                failed: false,
            })),
        }
    }

    /// Reserves the complete body envelope before sending an Internet request.
    ///
    /// # Errors
    /// Returns an error when the cumulative allowance or durable accounting is unavailable.
    pub(super) fn reserve(&self, maximum_bytes: u64) -> Result<InternetReservation> {
        let mut state = self.lock();
        ensure!(!state.failed, "Internet accounting is unavailable");
        let reserved = state
            .usage
            .reserved_bytes
            .checked_add(maximum_bytes)
            .context("Internet reservation overflow")?;
        let total = state
            .usage
            .charged_bytes
            .checked_add(reserved)
            .context("Internet usage overflow")?;
        if let InternetDataLimit::Bytes(limit) = state.limit {
            ensure!(total <= limit, "cumulative Internet allowance exhausted");
        }
        state.usage.reserved_bytes = reserved;
        state.persist()?;
        Ok(InternetReservation::new(self.clone(), maximum_bytes))
    }

    fn settle(&self, reserved: u64, charged: u64) -> Result<()> {
        let mut state = self.lock();
        state.usage.reserved_bytes = state
            .usage
            .reserved_bytes
            .checked_sub(reserved)
            .context("Internet reservation was already released")?;
        state.usage.charged_bytes = state
            .usage
            .charged_bytes
            .checked_add(charged)
            .context("Internet usage overflow")?;
        state.persist()
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl State {
    fn persist(&mut self) -> Result<()> {
        if let Some(disk) = &self.disk {
            if let Err(error) = disk.save(self.usage) {
                self.failed = true;
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(any(test, feature = "test"))]
mod test_support;
