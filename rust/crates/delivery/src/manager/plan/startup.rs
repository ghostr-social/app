use ghostr_engine::host_stats::HostStats;
use ghostr_engine::playback::{AdaptiveBufferPolicy, MediaConsumption, NetworkConditions};
use std::time::Duration;

#[derive(Clone, Copy)]
pub(crate) struct StartupContext {
    bitrate_bits_per_second: u64,
    observed_at_ms: u64,
    default_seconds: u64,
}

impl StartupContext {
    pub(crate) fn new(
        bitrate_bits_per_second: u64,
        observed_at_ms: u64,
        default_seconds: u64,
    ) -> Self {
        Self {
            bitrate_bits_per_second,
            observed_at_ms,
            default_seconds,
        }
    }
}

pub(crate) fn startup_seconds(stats: &HostStats, host: &str, context: StartupContext) -> u64 {
    let estimate = stats
        .host_throughput(host)
        .or_else(|| stats.overall_throughput());
    let Some(estimate) = estimate else {
        return context.default_seconds;
    };
    let ttfb = stats
        .expected_ttfb(host)
        .unwrap_or(Duration::from_millis(250));
    let network = NetworkConditions::from_estimate(estimate, ttfb, context.observed_at_ms);
    AdaptiveBufferPolicy::default()
        .target(
            network,
            MediaConsumption::new(context.bitrate_bits_per_second, 1_000),
        )
        .startup()
        .as_secs()
}
