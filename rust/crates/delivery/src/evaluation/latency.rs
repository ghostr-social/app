use super::types::LatencyDistribution;
use std::collections::VecDeque;

const SAMPLE_CAPACITY: usize = 2_048;

#[derive(Default)]
pub(super) struct LatencySamples {
    values: VecDeque<u64>,
}

impl LatencySamples {
    pub(super) fn push(&mut self, value: u64) {
        if self.values.len() == SAMPLE_CAPACITY {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    pub(super) fn distribution(&self) -> LatencyDistribution {
        let mut values: Vec<_> = self.values.iter().copied().collect();
        values.sort_unstable();
        LatencyDistribution {
            samples: values.len() as u64,
            p50_ms: percentile(&values, 50),
            p95_ms: percentile(&values, 95),
            p99_ms: percentile(&values, 99),
        }
    }
}

fn percentile(values: &[u64], percent: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let rank = percent.saturating_mul(values.len()).div_ceil(100);
    values[rank.saturating_sub(1).min(values.len() - 1)]
}
