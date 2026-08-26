mod metrics;
mod playback;

use super::latency::LatencySamples;
use super::privacy::EvaluationPrivacy;
use super::types::EvaluationSnapshot;
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;
use std::collections::{BTreeMap, HashSet};

#[derive(Default)]
pub struct EvaluationTracker {
    metrics: EvaluationSnapshot,
    privacy: EvaluationPrivacy,
    active: Option<ActivePlayback>,
    first_frame_latency: LatencySamples,
    replenish_latency: LatencySamples,
    recovery_latency: LatencySamples,
    unused_full_downloads: HashSet<PostId>,
    first_frame_quality_sum: u128,
    first_frame_quality_count: u64,
    watch_quality_sum: u128,
    played_ms: u64,
    budget_network_error_sum: i128,
    budget_storage_error_sum: i128,
    shadow_delta_sum: u128,
    budget_qoe_sum: u128,
    matched_network_bytes: u128,
    matched_storage_byte_ms: u128,
    readiness_observations: u64,
    readiness_expected: u128,
    readiness_observed: u64,
    adaptation_success_expected: u128,
    adaptation_success_observed: u64,
    adaptation_latency_correct: [u64; 3],
    adaptation_windows: BTreeMap<String, AdaptationWindow>,
    last_shadow_price_total: Option<u64>,
    last_storage_observation: Option<(u64, u64)>,
    last_readiness_at_ms: Option<u64>,
    readiness_underflow_active: bool,
    readiness_underflow_started_at_ms: Option<u64>,
}

struct ActivePlayback {
    post: PostId,
    focused_at_ms: u64,
    last_at_ms: u64,
    phase: PlaybackPhase,
    bitrate_bps: u64,
    presented: bool,
    startup_failure_counted: bool,
}

struct AdaptationWindow {
    started_at_ms: Option<u64>,
    last_seen_at_ms: u64,
}

impl EvaluationTracker {
    pub(crate) fn snapshot(&self) -> EvaluationSnapshot {
        let mut output = self.metrics.clone();
        output.user_visible.swipe_to_first_frame = self.first_frame_latency.distribution();
        output.readiness.replenish_after_burst = self.replenish_latency.distribution();
        output.adaptation.recovery_after_change = self.recovery_latency.distribution();
        output.efficiency.full_downloads_never_useful = output
            .efficiency
            .full_downloads_never_useful
            .saturating_add(self.unused_full_downloads.len() as u64);
        self.populate_derived(&mut output);
        output
    }

    fn populate_derived(&self, output: &mut EvaluationSnapshot) {
        playback::populate(self, output);
        metrics::populate(self, output);
    }
}

fn ratio(numerator: u64, denominator: u64) -> u16 {
    if denominator == 0 {
        return 0;
    }
    (u128::from(numerator)
        .saturating_mul(10_000)
        .checked_div(u128::from(denominator))
        .unwrap_or_default()
        .min(10_000)) as u16
}

fn average(sum: i128, count: u64) -> i32 {
    if count == 0 {
        return 0;
    }
    (sum / i128::from(count)).clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
}

fn unit_rate(value: u128, resource: u128) -> u64 {
    if resource == 0 {
        return 0;
    }
    value.saturating_mul(1_000_000).saturating_div(resource) as u64
}
