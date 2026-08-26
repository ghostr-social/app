use super::EvidenceField;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

mod posterior;
mod types;
use posterior::{Level, Posterior};
pub use types::{CalibrationContext, CalibrationDimensions, CalibrationLabel, ReliabilityEstimate};
const LABEL_CAPACITY: usize = 4_096;
const DEFAULT_HALF_LIFE_MS: u64 = 24 * 60 * 60 * 1_000;
const PRIOR_WEIGHT: f64 = 0.25;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FieldReliabilityModel {
    labels: VecDeque<CalibrationLabel>,
    capacity: usize,
    half_life_ms: u64,
}

impl Default for FieldReliabilityModel {
    fn default() -> Self {
        Self {
            labels: VecDeque::new(),
            capacity: LABEL_CAPACITY,
            half_life_ms: DEFAULT_HALF_LIFE_MS,
        }
    }
}

impl FieldReliabilityModel {
    pub fn observe(&mut self, label: CalibrationLabel) {
        if self.labels.len() == self.capacity {
            self.labels.pop_front();
        }
        self.labels.push_back(label);
    }

    pub fn estimate(&self, wanted: &CalibrationContext, now_ms: u64) -> ReliabilityEstimate {
        let mut posterior = Posterior::uniform();
        for level in Level::ALL {
            posterior = self.level(posterior, wanted, now_ms, level);
        }
        posterior.estimate()
    }

    /// Restores and normalizes a persisted reliability model.
    ///
    /// # Errors
    ///
    /// Returns a JSON decoding error when the persisted model is malformed.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self::normalized(serde_json::from_str(json)?))
    }

    pub(crate) fn normalized(mut model: Self) -> Self {
        model.capacity = model.capacity.clamp(1, LABEL_CAPACITY);
        model.half_life_ms = model.half_life_ms.max(1);
        for label in &mut model.labels {
            label.weight_bps = label.weight_bps.min(10_000);
        }
        while model.labels.len() > model.capacity {
            model.labels.pop_front();
        }
        model
    }

    fn level(
        &self,
        parent: Posterior,
        wanted: &CalibrationContext,
        now_ms: u64,
        level: Level,
    ) -> Posterior {
        let mut success = 0.0;
        let mut failure = 0.0;
        for label in self
            .labels
            .iter()
            .filter(|label| level.matches(label, wanted))
        {
            let weight = self.weight(label, now_ms);
            if label.correct {
                success += weight;
            } else {
                failure += weight;
            }
        }
        Posterior {
            alpha: parent.mean() * PRIOR_WEIGHT + success,
            beta: (1.0 - parent.mean()) * PRIOR_WEIGHT + failure,
            samples: parent.samples * PRIOR_WEIGHT + success + failure,
        }
    }

    fn weight(&self, label: &CalibrationLabel, now_ms: u64) -> f64 {
        let age = now_ms.saturating_sub(label.observed_at_ms);
        let decay = 0.5_f64.powf(age as f64 / self.half_life_ms as f64);
        decay * f64::from(label.weight_bps) / 10_000.0
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "calibration/test_support.rs"]
mod test_support;
