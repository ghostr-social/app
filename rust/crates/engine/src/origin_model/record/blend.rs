use super::{ErrorReason, Quantiles};
use std::collections::BTreeMap;

pub(super) fn mix(long: f64, short: f64, weight: f64) -> f64 {
    long * (1.0 - weight) + short * weight
}

pub(super) fn blend_quantiles(
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

pub(super) fn blend_errors(
    mut long: BTreeMap<ErrorReason, f64>,
    short: &BTreeMap<ErrorReason, f64>,
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
