use super::probability::decay;
use serde::{Deserialize, Serialize};

const CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Sample {
    value: u64,
    at_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct DecayedQuantiles {
    samples: Vec<Sample>,
}

impl DecayedQuantiles {
    pub fn observe(&mut self, value: u64, at_ms: u64) {
        if self.samples.len() == CAPACITY {
            self.samples.remove(0);
        }
        self.samples.push(Sample { value, at_ms });
    }

    pub fn summary(&self, at_ms: u64, half_life_ms: u64) -> Option<Quantiles> {
        let mut samples: Vec<_> = self
            .samples
            .iter()
            .map(|sample| {
                (
                    sample.value,
                    decay(at_ms.saturating_sub(sample.at_ms), half_life_ms),
                )
            })
            .collect();
        samples.sort_by_key(|sample| sample.0);
        let weight: f64 = samples.iter().map(|sample| sample.1).sum();
        (weight > f64::EPSILON).then(|| Quantiles {
            p10: percentile(&samples, weight, 0.10),
            p50: percentile(&samples, weight, 0.50),
            p90: percentile(&samples, weight, 0.90),
            p95: percentile(&samples, weight, 0.95),
            p99: percentile(&samples, weight, 0.99),
            evidence: weight,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Quantiles {
    pub p10: u64,
    pub p50: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
    pub evidence: f64,
}

fn percentile(samples: &[(u64, f64)], total: f64, quantile: f64) -> u64 {
    let target = total * quantile;
    let mut cumulative = 0.0;
    for (value, weight) in samples {
        cumulative += weight;
        if cumulative >= target {
            return *value;
        }
    }
    samples.last().map_or(0, |sample| sample.0)
}
