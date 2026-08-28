use super::super::TwinState;
use crate::adaptive::ActionNode;

pub(super) fn completion_ms(
    state: TwinState,
    action: &ActionNode,
    quantile_bps: u16,
    succeeds: bool,
) -> u64 {
    let times = action.forecast.completion;
    let fallback = state
        .rtt_ms
        .saturating_mul(u64::from(action.resources.requests > 0))
        .saturating_add(
            action.resources.network_bytes.saturating_mul(8_000) / state.throughput_bps.max(1),
        );
    let expected = times.expected_ms.max(fallback);
    let sampled = interpolate(
        expected,
        times.p95_ms.max(expected),
        times.p99_ms.max(expected),
        quantile_bps,
    );
    if succeeds && action.resources.requests <= state.request_slots {
        sampled
    } else {
        sampled.max(times.cvar_ms).saturating_mul(2)
    }
}

fn interpolate(expected: u64, p95: u64, p99: u64, quantile: u16) -> u64 {
    match quantile {
        0..=4_999 => expected.saturating_mul(50 + u64::from(quantile) / 100) / 100,
        5_000..=9_499 => expected + (p95 - expected) * u64::from(quantile - 5_000) / 4_500,
        _ => p95 + (p99 - p95) * u64::from(quantile - 9_500) / 500,
    }
}
