use super::{DeliveryState, MediaConsumption};
use ghostr_engine::host_stats::{HostStats, OPTIMISTIC_THROUGHPUT_BPS};
use ghostr_engine::playback::{AdaptiveBufferPolicy, NetworkConditions};
use std::time::Duration;

#[derive(Clone, Copy)]
pub(super) struct Targets {
    pub(super) steady: Duration,
    pub(super) emergency: Duration,
    pub(super) inflow_bits_per_second: u64,
}

pub(super) struct TargetInputs<'a> {
    pub(super) stats: &'a HostStats,
    pub(super) host: Option<&'a str>,
    pub(super) media: MediaConsumption,
    pub(super) observed_at_ms: u64,
}

pub(super) fn targets(state: &DeliveryState, inputs: TargetInputs<'_>) -> Targets {
    let estimate = inputs
        .host
        .and_then(|value| inputs.stats.host_throughput(value))
        .or_else(|| inputs.stats.overall_throughput());
    let inflow = estimate
        .map(|value| value.bytes_per_second())
        .unwrap_or(OPTIMISTIC_THROUGHPUT_BPS);
    let Some(estimate) = estimate else {
        return fallback_targets(state, inflow);
    };
    let ttfb = inputs
        .host
        .and_then(|value| inputs.stats.expected_ttfb(value))
        .or_else(|| inputs.stats.overall_ttfb())
        .unwrap_or(Duration::from_millis(250));
    let target = AdaptiveBufferPolicy::default().target(
        NetworkConditions::from_estimate(estimate, ttfb, inputs.observed_at_ms),
        inputs.media,
    );
    Targets {
        steady: target.steady(),
        emergency: target.emergency_horizon(),
        inflow_bits_per_second: finite_bits(inflow),
    }
}

fn fallback_targets(state: &DeliveryState, inflow: f64) -> Targets {
    let params = state.params();
    Targets {
        steady: Duration::from_secs(
            params
                .emergency_buffer_s
                .max(params.head_seconds.saturating_mul(2)),
        ),
        emergency: Duration::from_secs(params.emergency_buffer_s),
        inflow_bits_per_second: finite_bits(inflow),
    }
}

fn finite_bits(bytes_per_second: f64) -> u64 {
    if !bytes_per_second.is_finite() || bytes_per_second <= 0.0 {
        return 0;
    }
    (bytes_per_second * 8.0).min(u64::MAX as f64).round() as u64
}
