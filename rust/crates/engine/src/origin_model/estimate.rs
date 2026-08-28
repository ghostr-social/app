use super::quantile::Quantiles;
use super::{ErrorReason, OriginContext, OriginEnvironment, RecordSnapshot};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionMode {
    Normal,
    Safety,
    Emergency,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdaptationState {
    Long,
    Short,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbabilityEstimate {
    pub mean: f64,
    pub lower: f64,
    pub upper: f64,
    pub selected: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuantileEstimate {
    pub p10: u64,
    pub p50: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
    pub selected: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OriginEstimate {
    pub context: OriginContext,
    pub(crate) environment: OriginEnvironment,
    pub success: ProbabilityEstimate,
    pub range_compliance: Option<ProbabilityEstimate>,
    pub ttfb_ms: QuantileEstimate,
    pub throughput_bps: QuantileEstimate,
    error_frequencies: BTreeMap<ErrorReason, f64>,
    pub effective_samples: f64,
    pub adaptation: AdaptationState,
    pub uncertainty: f64,
}

pub(super) struct EstimateInput {
    pub context: OriginContext,
    pub environment: OriginEnvironment,
    pub snapshot: RecordSnapshot,
    pub prior: super::ColdStartPrior,
    pub mode: DecisionMode,
    pub success_prior_evidence: f64,
}

pub(super) fn build_estimate(input: EstimateInput) -> OriginEstimate {
    let EstimateInput {
        context,
        environment,
        snapshot,
        prior,
        mode,
        success_prior_evidence,
    } = input;
    let uncertainty = sample_uncertainty(snapshot.evidence);
    let success_uncertainty = sample_uncertainty(snapshot.evidence + success_prior_evidence);
    let range_uncertainty = sample_uncertainty(snapshot.range_evidence);
    let success = probability(snapshot.success_mean, success_uncertainty, mode);
    let range = probability(snapshot.range_mean, range_uncertainty, mode);
    let ttfb = snapshot.ttfb.unwrap_or_else(|| prior_ttfb(prior));
    let throughput = snapshot
        .throughput
        .unwrap_or_else(|| prior_throughput(prior));
    OriginEstimate {
        context,
        environment,
        success,
        range_compliance: method_uses_ranges(context.method).then_some(range),
        ttfb_ms: latency_estimate(ttfb, uncertainty, mode),
        throughput_bps: throughput_estimate(throughput, uncertainty, mode),
        error_frequencies: snapshot.errors,
        effective_samples: snapshot.evidence,
        adaptation: if snapshot.adapting {
            AdaptationState::Short
        } else {
            AdaptationState::Long
        },
        uncertainty,
    }
}

fn sample_uncertainty(evidence: f64) -> f64 {
    (1.0 / (evidence + 1.0).sqrt()).clamp(0.0, 1.0)
}

#[cfg(test)]
#[path = "estimate/test_support.rs"]
mod test_support;

fn probability(mean: f64, uncertainty: f64, mode: DecisionMode) -> ProbabilityEstimate {
    let z = match mode {
        DecisionMode::Normal => 0.0,
        DecisionMode::Safety => 1.645,
        DecisionMode::Emergency => 2.326,
    };
    let margin = (uncertainty * 0.25).min(0.49);
    let lower = (mean - margin).clamp(0.0, 1.0);
    let upper = (mean + margin).clamp(0.0, 1.0);
    let selected = (mean - z * margin).clamp(0.0, 1.0);
    ProbabilityEstimate {
        mean,
        lower,
        upper,
        selected,
    }
}

fn latency_estimate(value: Quantiles, uncertainty: f64, mode: DecisionMode) -> QuantileEstimate {
    let base = match mode {
        DecisionMode::Normal => value.p50,
        DecisionMode::Safety => value.p95,
        DecisionMode::Emergency => value.p99,
    };
    let factor = 1.0
        + uncertainty
            * match mode {
                DecisionMode::Normal => 0.0,
                DecisionMode::Safety => 0.5,
                DecisionMode::Emergency => 1.0,
            };
    quantile_estimate(value, (base as f64 * factor).round() as u64)
}

fn throughput_estimate(value: Quantiles, uncertainty: f64, mode: DecisionMode) -> QuantileEstimate {
    let base = match mode {
        DecisionMode::Normal => value.p50,
        DecisionMode::Safety | DecisionMode::Emergency => value.p10,
    };
    let penalty = match mode {
        DecisionMode::Normal => 1.0,
        DecisionMode::Safety => 1.0 - 0.5 * uncertainty,
        DecisionMode::Emergency => 1.0 - 0.75 * uncertainty,
    };
    quantile_estimate(value, (base as f64 * penalty.max(0.1)).round() as u64)
}

fn quantile_estimate(value: Quantiles, selected: u64) -> QuantileEstimate {
    QuantileEstimate {
        p10: value.p10,
        p50: value.p50,
        p90: value.p90,
        p95: value.p95,
        p99: value.p99,
        selected: selected.max(1),
    }
}

fn prior_ttfb(prior: super::ColdStartPrior) -> Quantiles {
    let p50 = prior.ttfb_p50_ms;
    Quantiles {
        p10: p50 / 2,
        p50,
        p90: p50 * 2,
        p95: p50 * 3,
        p99: p50 * 5,
        evidence: 0.0,
    }
}

fn prior_throughput(prior: super::ColdStartPrior) -> Quantiles {
    let p50 = prior.throughput_p50_bps;
    Quantiles {
        p10: p50 / 4,
        p50,
        p90: p50 * 2,
        p95: p50 * 2,
        p99: p50 * 2,
        evidence: 0.0,
    }
}

fn method_uses_ranges(method: super::RequestMethod) -> bool {
    matches!(
        method,
        super::RequestMethod::PrefixGet
            | super::RequestMethod::TailGet
            | super::RequestMethod::RangeGet
    )
}
