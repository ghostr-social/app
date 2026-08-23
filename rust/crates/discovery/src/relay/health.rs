//! Bounded relay-read circuits with fenced, single-probe recovery.

use book::HealthBook;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::time::Instant;

mod book;
mod circuit;

pub(super) const INITIAL_BACKOFF: Duration = Duration::from_secs(2);
pub(super) const MAX_BACKOFF: Duration = Duration::from_secs(300);
pub(super) const PROBE_LEASE: Duration = Duration::from_secs(30);
pub(super) const CIRCUIT_CAPACITY: usize = 256;
pub(super) const RECOVERY_PROBES_PER_BATCH: usize = 1;
pub(super) const ACTIVE_RECOVERY_PROBE_LIMIT: usize = 4;

#[derive(Clone, Debug)]
pub(crate) struct RelayAdmission {
    pub(super) url: String,
    pub(super) generation: u64,
}

pub(crate) struct RelayAdmissionBatch {
    health: Arc<RelayHealth>,
    admissions: Vec<RelayAdmission>,
    all_candidates_admitted: bool,
    settled: bool,
}

#[derive(Default)]
pub(crate) struct RelayHealth {
    book: Mutex<HealthBook>,
}

impl RelayAdmission {
    pub(crate) fn url(&self) -> &str {
        &self.url
    }
}

impl RelayAdmissionBatch {
    pub(crate) fn urls(&self) -> Vec<String> {
        self.admissions
            .iter()
            .map(|admission| admission.url.clone())
            .collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.admissions.is_empty()
    }

    pub(crate) fn covers(&self, relays: &[String]) -> bool {
        self.all_candidates_admitted
            && self.admissions.len() == relays.len()
            && self
                .admissions
                .iter()
                .all(|admission| relays.contains(&admission.url))
    }

    pub(crate) fn settle(&mut self, completed: &[String], failed: &[String]) {
        self.health.observe(&self.admissions, completed, failed);
        self.settled = true;
    }
}

impl Drop for RelayAdmissionBatch {
    fn drop(&mut self) {
        if !self.settled {
            self.health.release(&self.admissions);
        }
    }
}

impl RelayHealth {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn batch(self: &Arc<Self>, candidates: &[String]) -> RelayAdmissionBatch {
        let admissions = self.admit(candidates);
        RelayAdmissionBatch {
            health: self.clone(),
            all_candidates_admitted: admissions.len() == candidates.len(),
            admissions,
            settled: false,
        }
    }

    pub(crate) fn admit(&self, candidates: &[String]) -> Vec<RelayAdmission> {
        self.lock().admit(candidates, Instant::now())
    }

    pub(crate) fn observe(
        &self,
        admissions: &[RelayAdmission],
        completed: &[String],
        failed: &[String],
    ) {
        self.lock()
            .observe(admissions, completed, failed, Instant::now());
    }

    pub(crate) fn release(&self, admissions: &[RelayAdmission]) {
        self.lock().release(admissions, Instant::now());
    }

    pub(crate) fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> MutexGuard<'_, HealthBook> {
        self.book
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
