use super::{ConcurrencyEvidence, EvidenceWindow};

const TRIAL_SAMPLES: usize = 4;
const MINIMUM_GAIN_PERCENT: u64 = 15;
const MAXIMUM_TTFB_INFLATION_PERCENT: u64 = 40;

#[derive(Clone, Copy, Debug)]
pub(super) struct Trial {
    baseline: EvidenceWindow,
    evidence: EvidenceWindow,
    misses: usize,
}

pub(super) enum TrialProgress {
    Pending(Trial),
    Accepted(EvidenceWindow),
    Rejected,
    Abandoned,
}

impl Trial {
    pub(super) fn new(baseline: EvidenceWindow) -> Self {
        Self {
            baseline,
            evidence: EvidenceWindow::default(),
            misses: 0,
        }
    }

    pub(super) fn observe(mut self, evidence: ConcurrencyEvidence) -> TrialProgress {
        self.misses = 0;
        self.evidence.push(evidence);
        if self.evidence.len() < TRIAL_SAMPLES {
            return TrialProgress::Pending(self);
        }
        match trial_improved(self) {
            true => TrialProgress::Accepted(self.evidence),
            false => TrialProgress::Rejected,
        }
    }

    pub(super) fn miss(mut self) -> TrialProgress {
        self.misses = self.misses.saturating_add(1);
        match self.misses < TRIAL_SAMPLES {
            true => TrialProgress::Pending(self),
            false => TrialProgress::Abandoned,
        }
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
