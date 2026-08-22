use super::probability::decay;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ErrorReason {
    Timeout,
    Dns,
    Tls,
    Http4xx,
    Http5xx,
    RangeNoncompliant,
    InvalidResponse,
    Connection,
    Policy,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Count {
    value: f64,
    at_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct ErrorHistogram {
    counts: BTreeMap<ErrorReason, Count>,
}

impl ErrorHistogram {
    pub fn observe(&mut self, reason: ErrorReason, at_ms: u64, half_life_ms: u64) {
        let count = self
            .counts
            .entry(reason)
            .or_insert(Count { value: 0.0, at_ms });
        count.value *= decay(at_ms.saturating_sub(count.at_ms), half_life_ms);
        count.value += 1.0;
        count.at_ms = at_ms;
    }

    pub fn frequencies(&self, at_ms: u64, half_life_ms: u64) -> BTreeMap<ErrorReason, f64> {
        let mut weighted = BTreeMap::new();
        for (reason, count) in &self.counts {
            let value = count.value * decay(at_ms.saturating_sub(count.at_ms), half_life_ms);
            weighted.insert(*reason, value);
        }
        let total: f64 = weighted.values().sum();
        if total > f64::EPSILON {
            for value in weighted.values_mut() {
                *value /= total;
            }
        }
        weighted
    }
}
