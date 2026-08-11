//! Conservative, evidence-driven transfer concurrency.
//!
//! The policy only learns from saturated windows. A higher limit is a
//! temporary one-step trial until aggregate throughput proves a useful
//! gain without materially inflating request latency.

use std::time::Duration;

mod window;
use window::EvidenceWindow;

const LEARNING_SAMPLES: usize = 4;
const TRIAL_SAMPLES: usize = 4;
const RETRY_BACKOFF_SAMPLES: usize = 8;
const MINIMUM_GAIN_PERCENT: u64 = 15;
const MAXIMUM_TTFB_INFLATION_PERCENT: u64 = 40;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkSetback {
    None,
    Stall,
    Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcurrencyEvidence {
    pub aggregate_bytes_per_second: u64,
    pub active_transfers: usize,
    pub saturated: bool,
    pub ttfb: Duration,
    pub setback: NetworkSetback,
}

#[derive(Clone, Copy, Debug)]
struct Trial {
    baseline: EvidenceWindow,
    evidence: EvidenceWindow,
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

    pub fn accepted_limit(self) -> usize {
        self.accepted
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
        if evidence.setback != NetworkSetback::None {
            self.back_off();
        } else if self.is_capacity_sample(evidence) {
            self.observe_capacity(evidence);
        }
        self.limit
    }

    fn is_capacity_sample(&self, evidence: ConcurrencyEvidence) -> bool {
        evidence.saturated
            && evidence.active_transfers == self.limit
            && evidence.aggregate_bytes_per_second > 0
    }

    fn observe_capacity(&mut self, evidence: ConcurrencyEvidence) {
        match self.trial.take() {
            Some(trial) => self.observe_trial(trial, evidence),
            None => self.observe_baseline(evidence),
        }
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
        self.trial = Some(Trial {
            baseline: self.baseline,
            evidence: EvidenceWindow::default(),
        });
    }

    fn observe_trial(&mut self, mut trial: Trial, evidence: ConcurrencyEvidence) {
        trial.evidence.push(evidence);
        if trial.evidence.len() < TRIAL_SAMPLES {
            self.trial = Some(trial);
            return;
        }
        if trial_improved(trial) {
            self.accept_trial(trial.evidence);
        } else {
            self.reject_trial();
        }
    }

    fn accept_trial(&mut self, evidence: EvidenceWindow) {
        self.accepted = self.limit;
        self.baseline = evidence;
        self.trial = None;
    }

    fn reject_trial(&mut self) {
        self.limit = self.accepted;
        self.baseline = EvidenceWindow::default();
        self.trial = None;
        self.retry_backoff = RETRY_BACKOFF_SAMPLES;
    }

    fn back_off(&mut self) {
        self.accepted = self.accepted.saturating_sub(1).max(1);
        self.limit = self.accepted;
        self.baseline = EvidenceWindow::default();
        self.trial = None;
        self.retry_backoff = RETRY_BACKOFF_SAMPLES;
    }
}

fn trial_improved(trial: Trial) -> bool {
    let throughput = trial.evidence.throughput();
    let baseline = trial.baseline.throughput();
    let useful_gain =
        throughput.saturating_mul(100) >= baseline.saturating_mul(100 + MINIMUM_GAIN_PERCENT);
    useful_gain && latency_is_bounded(trial)
}

fn latency_is_bounded(trial: Trial) -> bool {
    let latency = trial.evidence.ttfb_micros();
    let baseline = trial.baseline.ttfb_micros();
    baseline == 0
        || latency.saturating_mul(100)
            <= baseline.saturating_mul(100 + MAXIMUM_TTFB_INFLATION_PERCENT)
}
