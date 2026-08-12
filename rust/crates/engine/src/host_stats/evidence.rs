use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::time::Duration;

/// Sample weight for failure and latency observations.
pub(crate) const EWMA_ALPHA: f64 = 0.066_967_008_463_192_6;
const THROUGHPUT_HALF_LIFE_MS: f64 = 5_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThroughputSample {
    bytes: u64,
    elapsed: Duration,
    observed_at_ms: u64,
}

impl ThroughputSample {
    pub fn new(
        bytes: u64,
        elapsed: Duration,
        observed_at_ms: u64,
        active_transfers: usize,
    ) -> Option<Self> {
        NonZeroUsize::new(active_transfers)?;
        Some(Self {
            bytes,
            elapsed: (!elapsed.is_zero()).then_some(elapsed)?,
            observed_at_ms,
        })
    }

    pub fn observed_at_ms(self) -> u64 {
        self.observed_at_ms
    }

    fn bytes_per_second(self) -> f64 {
        self.bytes as f64 / self.elapsed.as_secs_f64()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThroughputEstimate {
    bytes_per_second: f64,
    variability_bytes_per_second: f64,
    sample_count: u64,
    last_observed_at_ms: u64,
}

impl ThroughputEstimate {
    pub fn bytes_per_second(self) -> f64 {
        self.bytes_per_second
    }

    pub fn variability_bytes_per_second(self) -> f64 {
        self.variability_bytes_per_second
    }

    pub fn sample_count(self) -> u64 {
        self.sample_count
    }

    pub fn last_observed_at_ms(self) -> u64 {
        self.last_observed_at_ms
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct Ewma {
    value: Option<f64>,
}

impl Ewma {
    fn observe(&mut self, sample: f64) {
        self.observe_weighted(sample, EWMA_ALPHA);
    }

    fn observe_weighted(&mut self, sample: f64, weight: f64) {
        self.value = Some(self.value.map_or(sample, |previous| {
            weight * sample + (1.0 - weight) * previous
        }));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct HostRecord {
    throughput_bps: Ewma,
    ttfb_ms: Ewma,
    failure_ratio: Ewma,
    #[serde(default)]
    throughput_variance: f64,
    #[serde(default)]
    throughput_samples: u64,
    #[serde(default)]
    last_observed_at_ms: u64,
}

impl HostRecord {
    pub(super) fn observe_throughput(&mut self, sample: ThroughputSample) -> bool {
        if self.throughput_samples > 0 && sample.observed_at_ms < self.last_observed_at_ms {
            return false;
        }
        let rate = sample.bytes_per_second();
        let weight = self.throughput_weight(sample);
        self.update_variance(rate, weight);
        self.throughput_bps.observe_weighted(rate, weight);
        self.throughput_samples = self.throughput_samples.saturating_add(1);
        self.last_observed_at_ms = sample.observed_at_ms;
        true
    }

    pub(super) fn throughput(&self) -> Option<ThroughputEstimate> {
        Some(ThroughputEstimate {
            bytes_per_second: self.throughput_bps.value?,
            variability_bytes_per_second: self.throughput_variance.max(0.0).sqrt(),
            sample_count: self.throughput_samples,
            last_observed_at_ms: self.last_observed_at_ms,
        })
    }

    pub(super) fn observe_ttfb(&mut self, milliseconds: u64) {
        self.ttfb_ms.observe(milliseconds as f64);
    }

    pub(super) fn ttfb(&self) -> Option<Duration> {
        self.ttfb_ms
            .value
            .map(|value| Duration::from_millis(value.max(0.0).round() as u64))
    }

    pub(super) fn observe_failure(&mut self, failed: f64) {
        self.failure_ratio.observe(failed);
    }

    pub(super) fn failure_ratio(&self) -> Option<f64> {
        self.failure_ratio.value
    }

    pub(super) fn last_observed_at_ms(&self) -> u64 {
        self.last_observed_at_ms
    }

    pub(super) fn normalize_loaded(&mut self) {
        if self.throughput_bps.value.is_some() && self.throughput_samples == 0 {
            self.throughput_samples = 1;
        }
        self.throughput_variance = self.throughput_variance.max(0.0);
    }

    fn throughput_weight(&self, sample: ThroughputSample) -> f64 {
        let observed_elapsed = sample
            .observed_at_ms
            .saturating_sub(self.last_observed_at_ms);
        let elapsed_ms = match observed_elapsed {
            0 => sample.elapsed.as_secs_f64() * 1_000.0,
            value => value as f64,
        };
        1.0 - 0.5f64.powf(elapsed_ms / THROUGHPUT_HALF_LIFE_MS)
    }

    fn update_variance(&mut self, sample: f64, weight: f64) {
        let Some(previous) = self.throughput_bps.value else {
            self.throughput_variance = 0.0;
            return;
        };
        let difference = sample - previous;
        self.throughput_variance =
            (1.0 - weight) * (self.throughput_variance + weight * difference * difference);
    }
}
