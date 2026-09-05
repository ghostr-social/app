//! Bootstrap service scenarios for quality selection. Exact dependency arrivals
//! remain the adapter's responsibility; this estimate cannot certify readiness.
use super::{BufferScenario, EstimateConfidence, MediaConsumption, NetworkConditions};
use super::{PlaybackPhase, UsableArrival};
use core::time::Duration;

const HORIZON_MS: u64 = 20_000;
const COMPLETION_INTERVAL_MS: u64 = 1_000;

pub(super) fn predicted_requirement(
    network: NetworkConditions,
    media: MediaConsumption,
    remaining: Duration,
) -> Duration {
    let horizon_ms = horizon(media, remaining);
    let scenario = BufferScenario::new(
        horizon_ms,
        media.playback_rate_milli,
        PlaybackPhase::Playing,
    );
    let arrivals = predicted_arrivals(network, media, horizon_ms);
    let required = scenario
        .required_ms(&arrivals)
        .expect("bounded ordered bootstrap scenario");
    Duration::from_millis(required).min(remaining)
}

fn horizon(media: MediaConsumption, remaining: Duration) -> u64 {
    let rate = u128::from(media.playback_rate_milli.max(1));
    (remaining.as_millis().saturating_mul(1_000).div_ceil(rate)).min(u128::from(HORIZON_MS)) as u64
}

fn predicted_arrivals(
    network: NetworkConditions,
    media: MediaConsumption,
    horizon_ms: u64,
) -> Vec<UsableArrival> {
    let delay_ms = network.ttfb.as_millis().min(u128::from(HORIZON_MS)) as u64
        + processing_margin_ms(network.confidence);
    let service = u128::from(network.sustainable_bits_per_second());
    let bitrate = u128::from(media.bitrate_bits_per_second.max(1));
    (1..=horizon_ms / COMPLETION_INTERVAL_MS)
        .map(|second| {
            let at_ms = second * COMPLETION_INTERVAL_MS + delay_ms;
            let extent =
                (u128::from(second) * service * 1_000 / bitrate).min(u128::from(u64::MAX)) as u64;
            UsableArrival::new(at_ms, extent)
        })
        .collect()
}

pub(super) fn processing_margin_ms(confidence: EstimateConfidence) -> u64 {
    match confidence {
        EstimateConfidence::High => 250,
        EstimateConfidence::Medium => 2_000,
        EstimateConfidence::Low => 4_000,
    }
}
