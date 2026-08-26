use crate::host_stats::ThroughputEstimate;
use core::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EstimateConfidence {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkConditions {
    pub(super) bytes_per_second: u64,
    pub(super) variability_bytes_per_second: u64,
    pub(super) ttfb: Duration,
    pub(super) confidence: EstimateConfidence,
}

impl NetworkConditions {
    pub const fn new(
        bytes_per_second: u64,
        variability_bytes_per_second: u64,
        ttfb: Duration,
        confidence: EstimateConfidence,
    ) -> Self {
        Self {
            bytes_per_second,
            variability_bytes_per_second,
            ttfb,
            confidence,
        }
    }

    pub fn from_estimate(
        estimate: ThroughputEstimate,
        ttfb: Duration,
        observed_at_ms: u64,
    ) -> Self {
        Self::new(
            finite_u64(estimate.bytes_per_second()),
            finite_u64(estimate.variability_bytes_per_second()),
            ttfb,
            EstimateConfidence::from_evidence(
                estimate.sample_count(),
                estimate.last_observed_at_ms(),
                observed_at_ms,
            ),
        )
    }

    /// Throughput available after discounting observed variation according
    /// to the amount and freshness of supporting evidence.
    pub(crate) fn sustainable_bits_per_second(self) -> u64 {
        let throughput = self.bytes_per_second.saturating_mul(8);
        let variability = self.variability_bytes_per_second.saturating_mul(8);
        throughput.saturating_sub(discounted_variability(variability, self.confidence))
    }

    pub(crate) fn confidence(self) -> EstimateConfidence {
        self.confidence
    }
}

impl EstimateConfidence {
    pub fn from_evidence(sample_count: u64, sampled_at_ms: u64, now_ms: u64) -> Self {
        let age_ms = now_ms.saturating_sub(sampled_at_ms);
        if sample_count >= 8 && age_ms <= 60_000 {
            return Self::High;
        }
        if sample_count >= 3 && age_ms <= 300_000 {
            return Self::Medium;
        }
        Self::Low
    }
}

fn finite_u64(value: f64) -> u64 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.min(u64::MAX as f64).round() as u64
}

fn discounted_variability(variability: u64, confidence: EstimateConfidence) -> u64 {
    match confidence {
        EstimateConfidence::Low => variability.saturating_mul(2),
        EstimateConfidence::Medium => variability.saturating_mul(3) / 2,
        EstimateConfidence::High => variability,
    }
}
