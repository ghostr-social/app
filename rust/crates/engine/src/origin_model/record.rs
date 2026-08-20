use super::change::ChangeDetector;
use super::errors::ErrorHistogram;
use super::probability::{BetaPosterior, DiscountedBeta};
use super::quantile::{DecayedQuantiles, Quantiles};
use super::{ColdStartPrior, ErrorReason, ModelTiming, OriginObservation, OriginOutcome};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Statistics {
    success: DiscountedBeta,
    range: DiscountedBeta,
    ttfb: DecayedQuantiles,
    throughput: DecayedQuantiles,
    errors: ErrorHistogram,
}

impl Statistics {
    fn observe(&mut self, item: &OriginObservation, half_life_ms: u64) {
        let at = item.observed_at_ms;
        match item.outcome {
            OriginOutcome::Success => self.success.observe(true, at, half_life_ms),
            OriginOutcome::Failure(reason) => {
                self.success.observe(false, at, half_life_ms);
                self.errors.observe(reason, at, half_life_ms);
            }
            OriginOutcome::Cancelled => return,
        }
        if let Some(value) = item.range_compliant {
            self.range.observe(value, at, half_life_ms);
        }
        if let Some(value) = item.ttfb_ms {
            self.ttfb.observe(value, at);
        }
        if let Some(value) = item.throughput_bps {
            self.throughput.observe(value, at);
        }
    }

    fn snapshot(&self, prior: ColdStartPrior, now: u64, half_life: u64) -> StatSnapshot {
        StatSnapshot {
            success: self.success.posterior(
                prior.success_alpha,
                prior.success_beta,
                now,
                half_life,
            ),
            range: self
                .range
                .posterior(prior.range_alpha, prior.range_beta, now, half_life),
            ttfb: self.ttfb.summary(now, half_life),
            throughput: self.throughput.summary(now, half_life),
            errors: self.errors.frequencies(now, half_life),
        }
    }

    fn surprise(&self, item: &OriginObservation, half_life_ms: u64) -> f64 {
        let success = self
            .success
            .posterior(8.0, 2.0, item.observed_at_ms, half_life_ms)
            .mean();
        let outcome = match item.outcome {
            OriginOutcome::Failure(_) => success,
            OriginOutcome::Success => 1.0 - success,
            OriginOutcome::Cancelled => 0.0,
        };
        let ttfb = metric_surprise(
            item.ttfb_ms,
            self.ttfb.summary(item.observed_at_ms, half_life_ms),
        );
        let throughput = metric_surprise(
            item.throughput_bps,
            self.throughput.summary(item.observed_at_ms, half_life_ms),
        );
        outcome.max(ttfb).max(throughput)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct AdaptiveRecord {
    long: Statistics,
    short: Statistics,
    detector: ChangeDetector,
    last_at_ms: u64,
}

impl AdaptiveRecord {
    pub fn observe(&mut self, item: &OriginObservation, config: ModelTiming) {
        if item.outcome == OriginOutcome::Cancelled {
            return;
        }
        let surprise = self.long.surprise(item, config.long_ms);
        self.detector.observe(surprise, item.observed_at_ms);
        self.long.observe(item, config.long_ms);
        self.short.observe(item, config.short_ms);
        self.last_at_ms = self.last_at_ms.max(item.observed_at_ms);
    }

    pub fn snapshot(&self, prior: ColdStartPrior, now: u64, config: ModelTiming) -> RecordSnapshot {
        let long = self.long.snapshot(prior, now, config.long_ms);
        let short = self.short.snapshot(prior, now, config.short_ms);
        let weight = self.detector.short_weight(now, config.adaptation_ms);
        RecordSnapshot::blend(long, short, weight, self.adapting(now, config))
    }

    pub fn last_at_ms(&self) -> u64 {
        self.last_at_ms
    }

    fn adapting(&self, now: u64, config: ModelTiming) -> bool {
        self.detector.adapting(now, config.adaptation_ms)
    }
}

fn metric_surprise(value: Option<u64>, expected: Option<Quantiles>) -> f64 {
    let (Some(value), Some(expected)) = (value, expected) else {
        return 0.0;
    };
    let below = value.saturating_mul(2) < expected.p50;
    let above = value > expected.p50.saturating_mul(2);
    f64::from(below || above)
}

pub(super) struct StatSnapshot {
    success: BetaPosterior,
    range: BetaPosterior,
    ttfb: Option<Quantiles>,
    throughput: Option<Quantiles>,
    errors: BTreeMap<ErrorReason, f64>,
}

pub(super) struct RecordSnapshot {
    pub success_mean: f64,
    pub range_mean: f64,
    pub ttfb: Option<Quantiles>,
    pub throughput: Option<Quantiles>,
    pub errors: BTreeMap<ErrorReason, f64>,
    pub evidence: f64,
    pub adapting: bool,
}

impl RecordSnapshot {
    fn blend(long: StatSnapshot, short: StatSnapshot, weight: f64, adapting: bool) -> Self {
        Self {
            success_mean: mix(long.success.mean(), short.success.mean(), weight),
            range_mean: mix(long.range.mean(), short.range.mean(), weight),
            ttfb: blend_quantiles(long.ttfb, short.ttfb, weight),
            throughput: blend_quantiles(long.throughput, short.throughput, weight),
            errors: blend_errors(long.errors, short.errors, weight),
            evidence: mix(long.success.evidence, short.success.evidence, weight),
            adapting,
        }
    }
}

fn mix(long: f64, short: f64, weight: f64) -> f64 {
    long * (1.0 - weight) + short * weight
}

fn blend_quantiles(
    long: Option<Quantiles>,
    short: Option<Quantiles>,
    weight: f64,
) -> Option<Quantiles> {
    match (long, short) {
        (Some(long), Some(short)) => Some(Quantiles {
            p10: mix_u64(long.p10, short.p10, weight),
            p50: mix_u64(long.p50, short.p50, weight),
            p90: mix_u64(long.p90, short.p90, weight),
            p95: mix_u64(long.p95, short.p95, weight),
            p99: mix_u64(long.p99, short.p99, weight),
            evidence: mix(long.evidence, short.evidence, weight),
        }),
        (value, None) | (None, value) => value,
    }
}

fn mix_u64(long: u64, short: u64, weight: f64) -> u64 {
    mix(long as f64, short as f64, weight).round() as u64
}

fn blend_errors(
    mut long: BTreeMap<ErrorReason, f64>,
    short: BTreeMap<ErrorReason, f64>,
    weight: f64,
) -> BTreeMap<ErrorReason, f64> {
    for reason in long.keys().chain(short.keys()).copied().collect::<Vec<_>>() {
        let value = mix(
            *long.get(&reason).unwrap_or(&0.0),
            *short.get(&reason).unwrap_or(&0.0),
            weight,
        );
        long.insert(reason, value);
    }
    long
}
