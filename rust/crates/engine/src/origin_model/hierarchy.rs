use super::quantile::Quantiles;
use super::{AdaptiveRecord, ColdStartPrior, ErrorReason, ModelTiming, RecordSnapshot};
use std::collections::BTreeMap;

pub(super) fn aggregate(
    records: [Option<&AdaptiveRecord>; 3],
    prior: ColdStartPrior,
    now: u64,
    timing: ModelTiming,
) -> RecordSnapshot {
    let mut result = RecordSnapshot {
        success_mean: prior.success_alpha / (prior.success_alpha + prior.success_beta),
        range_mean: prior.range_alpha / (prior.range_alpha + prior.range_beta),
        ttfb: Some(prior_ttfb(prior)),
        throughput: Some(prior_throughput(prior)),
        errors: BTreeMap::new(),
        evidence: 0.0,
        adapting: false,
    };
    for (level, record) in records.into_iter().enumerate() {
        let Some(record) = record else { continue };
        apply(&mut result, record.snapshot(prior, now, timing), level);
    }
    result
}

fn apply(current: &mut RecordSnapshot, next: RecordSnapshot, level: usize) {
    let shrinkage = [8.0, 4.0, 2.0][level];
    let weight = next.evidence / (next.evidence + shrinkage);
    current.success_mean = mix(current.success_mean, next.success_mean, weight);
    current.range_mean = mix(current.range_mean, next.range_mean, weight);
    current.ttfb = combine_quantiles(current.ttfb, next.ttfb, weight);
    current.throughput = combine_quantiles(current.throughput, next.throughput, weight);
    combine_errors(&mut current.errors, &next.errors, weight);
    current.evidence += next.evidence * [0.15, 0.35, 0.50][level];
    current.adapting |= next.adapting;
}

fn combine_quantiles(
    current: Option<Quantiles>,
    next: Option<Quantiles>,
    weight: f64,
) -> Option<Quantiles> {
    match (current, next) {
        (Some(current), Some(next)) => Some(Quantiles {
            p10: mix_u64(current.p10, next.p10, weight),
            p50: mix_u64(current.p50, next.p50, weight),
            p90: mix_u64(current.p90, next.p90, weight),
            p95: mix_u64(current.p95, next.p95, weight),
            p99: mix_u64(current.p99, next.p99, weight),
            evidence: current.evidence + next.evidence,
        }),
        (None, Some(next)) => Some(next),
        (current, None) => current,
    }
}

fn combine_errors(
    current: &mut BTreeMap<ErrorReason, f64>,
    next: &BTreeMap<ErrorReason, f64>,
    weight: f64,
) {
    let reasons: Vec<_> = current.keys().chain(next.keys()).copied().collect();
    for reason in reasons {
        let value = mix(
            *current.get(&reason).unwrap_or(&0.0),
            *next.get(&reason).unwrap_or(&0.0),
            weight,
        );
        current.insert(reason, value);
    }
}

fn mix(left: f64, right: f64, weight: f64) -> f64 {
    left * (1.0 - weight) + right * weight
}

fn mix_u64(left: u64, right: u64, weight: f64) -> u64 {
    mix(left as f64, right as f64, weight).round() as u64
}

fn prior_ttfb(prior: ColdStartPrior) -> Quantiles {
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

fn prior_throughput(prior: ColdStartPrior) -> Quantiles {
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
