use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{NetworkSnapshot, OriginHealth};
use ghostr_engine::host_stats::{host_of, OPTIMISTIC_THROUGHPUT_BPS};
use ghostr_engine::playback::EstimateConfidence;
use ghostr_engine::PostId;

pub(super) fn origins(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    post: &PostId,
) -> Vec<OriginHealth> {
    let Some(entry) = state.catalog().lookup(post) else {
        return Vec::new();
    };
    inputs
        .retry
        .live_urls(post, &entry.meta.urls)
        .into_iter()
        .filter_map(|url| origin(inputs, url))
        .collect()
}

fn origin(inputs: &PlanInputs<'_>, url: String) -> Option<OriginHealth> {
    let host = host_of(&url)?;
    let failure = (inputs.stats.failure_ratio(&host) * 10_000.0).round() as u16;
    Some(OriginHealth {
        source: url,
        available: true,
        throughput_bps: finite_bits(inputs.stats.expected_throughput(&host)),
        rtt_ms: inputs
            .stats
            .expected_ttfb(&host)
            .map_or(250, |value| value.as_millis() as u64),
        packet_loss_bps: 0,
        failure_bps: failure,
    })
}

pub(super) fn network(inputs: &PlanInputs<'_>) -> NetworkSnapshot {
    let estimate = inputs.stats.overall_throughput();
    let throughput = estimate.map_or(OPTIMISTIC_THROUGHPUT_BPS, |value| value.bytes_per_second());
    NetworkSnapshot {
        throughput_bps: finite_bits(throughput),
        rtt_ms: inputs
            .stats
            .overall_ttfb()
            .map_or(250, |value| value.as_millis() as u64),
        packet_loss_bps: inputs.packet_loss_bps,
        connection_capacity: inputs.connection_capacity.max(1),
        connection_ceiling: inputs.connection_ceiling.max(1),
        confidence: estimate.map_or(EstimateConfidence::Low, |value| {
            EstimateConfidence::from_evidence(
                value.sample_count(),
                value.last_observed_at_ms(),
                inputs.observed_at_ms,
            )
        }),
    }
}

fn finite_bits(bytes_per_second: f64) -> u64 {
    if !bytes_per_second.is_finite() || bytes_per_second <= 0.0 {
        return 0;
    }
    (bytes_per_second * 8.0).min(u64::MAX as f64).round() as u64
}
