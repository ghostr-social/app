//! Conservative, evidence-driven transfer concurrency.
//!
//! The policy opens trials only from demand-saturated windows. A higher
//! limit stays temporary until filled traffic proves a useful gain without
//! materially inflating request latency; an unclaimed trial is abandoned.

use std::time::Duration;

mod occupancy;
pub use occupancy::ConcurrencyOccupancy;
mod trial;
use trial::{Trial, TrialProgress};
mod window;
use window::EvidenceWindow;

const LEARNING_SAMPLES: usize = 4;
const RETRY_BACKOFF_SAMPLES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkSetback {
    None,
    Stall,
    Failure,
    SevereLoss,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcurrencyEvidence {
    pub aggregate_bytes_per_second: u64,
    pub occupancy: ConcurrencyOccupancy,
    pub saturated: bool,
    pub ttfb: Duration,
    pub setback: NetworkSetback,
}

#[derive(Clone, Copy, Debug)]
pub struct AdaptiveConcurrency {
    accepted: usize,
    limit: usize,
    maximum: usize,
    baseline: EvidenceWindow,
    trial: Option<Trial>,
    retry_backoff: usize,
}

impl AdaptiveConcurrency {
    pub fn new(initial: usize, maximum: usize) -> Self {
        let maximum = maximum.max(1);
        let accepted = initial.clamp(1, maximum);
        Self {
            accepted,
            limit: accepted,
            maximum,
            baseline: EvidenceWindow::default(),
            trial: None,
            retry_backoff: 0,
        }
    }

    pub fn limit(self) -> usize {
        self.limit
    }

    pub fn set_maximum(&mut self, maximum: usize) {
        self.maximum = maximum.max(1);
        if self.accepted <= self.maximum && self.limit <= self.maximum {
            return;
        }
        self.accepted = self.accepted.min(self.maximum);
        self.limit = self.accepted;
        self.baseline = EvidenceWindow::default();
        self.trial = None;
    }

    pub fn observe(&mut self, evidence: ConcurrencyEvidence) -> usize {
        match evidence.setback {
            NetworkSetback::SevereLoss => self.back_off_to_minimum(),
            NetworkSetback::Stall | NetworkSetback::Failure => self.back_off(),
            NetworkSetback::None => self.observe_healthy(evidence),
        }
        self.limit
    }

    fn observe_healthy(&mut self, evidence: ConcurrencyEvidence) {
        match self.trial.take() {
            Some(trial) => self.observe_trial_window(trial, evidence),
            None if self.is_capacity_sample(evidence) => self.observe_baseline(evidence),
            None => {}
        }
    }

    fn is_capacity_sample(&self, evidence: ConcurrencyEvidence) -> bool {
        evidence.saturated
            && evidence.occupancy.fills(self.limit)
            && evidence.aggregate_bytes_per_second > 0
    }

    fn observe_trial_window(&mut self, trial: Trial, evidence: ConcurrencyEvidence) {
        let progress = if evidence.aggregate_bytes_per_second > 0
            && evidence.occupancy.fills_trial(self.limit)
        {
            trial.observe(evidence)
        } else if evidence.occupancy.claims(self.limit) {
            TrialProgress::Pending(trial)
        } else {
            trial.miss()
        };
        self.apply_trial(progress);
    }

    fn observe_baseline(&mut self, evidence: ConcurrencyEvidence) {
        self.baseline.push(evidence);
        self.retry_backoff = self.retry_backoff.saturating_sub(1);
        if self.can_trial() {
            self.start_trial();
        }
    }

    fn can_trial(&self) -> bool {
        self.accepted < self.maximum
            && self.retry_backoff == 0
            && self.baseline.len() >= LEARNING_SAMPLES
    }

    fn start_trial(&mut self) {
        self.limit = self.accepted + 1;
        self.trial = Some(Trial::new(self.baseline));
    }

    fn apply_trial(&mut self, progress: TrialProgress) {
        match progress {
            TrialProgress::Pending(trial) => self.trial = Some(trial),
            TrialProgress::Accepted(evidence) => self.accept_trial(evidence),
            TrialProgress::Rejected => self.reject_trial(),
            TrialProgress::Abandoned => self.abandon_trial(),
        }
    }

    fn accept_trial(&mut self, evidence: EvidenceWindow) {
        self.accepted = self.limit;
        self.baseline = evidence;
        self.trial = None;
    }

    fn reject_trial(&mut self) {
        self.abandon_trial();
        self.retry_backoff = RETRY_BACKOFF_SAMPLES;
    }

    fn abandon_trial(&mut self) {
        self.limit = self.accepted;
        self.baseline = EvidenceWindow::default();
        self.trial = None;
    }

    fn back_off(&mut self) {
        self.accepted = self.accepted.saturating_sub(1).max(1);
        self.limit = self.accepted;
        self.baseline = EvidenceWindow::default();
        self.trial = None;
        self.retry_backoff = RETRY_BACKOFF_SAMPLES;
    }

    fn back_off_to_minimum(&mut self) {
        self.accepted = 1;
        self.limit = 1;
        self.baseline = EvidenceWindow::default();
        self.trial = None;
        self.retry_backoff = RETRY_BACKOFF_SAMPLES;
    }
}
