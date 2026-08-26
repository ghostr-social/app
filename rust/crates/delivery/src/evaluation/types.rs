use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvaluationSnapshot {
    pub(crate) user_visible: UserVisibleMetrics,
    pub efficiency: EfficiencyMetrics,
    pub budget: BudgetMetrics,
    pub(crate) readiness: ReadinessMetrics,
    pub(crate) adaptation: AdaptationMetrics,
    pub(crate) semantics: SemanticsMetrics,
    pub(super) integrity: IntegrityMetrics,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LatencyDistribution {
    pub(crate) samples: u64,
    pub(crate) p50_ms: u64,
    pub(crate) p95_ms: u64,
    pub(crate) p99_ms: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserVisibleMetrics {
    pub(crate) swipe_to_first_frame: LatencyDistribution,
    pub(super) startup_sessions: u64,
    pub(crate) startup_failures: u64,
    pub(super) startup_failure_rate_bps: u16,
    pub(super) stall_events: u64,
    pub(super) stall_ms: u64,
    pub(super) stall_ratio_bps: u16,
    pub(crate) first_frame_quality_bps: u64,
    pub(super) watch_weighted_quality_bps: u64,
    pub(super) quality_discontinuities: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub(super) total_bytes: u64,
    pub(super) useful_watched_bytes: u64,
    pub aborted_bytes: u64,
    pub(super) duplicate_hedge_bytes: u64,
    pub(crate) completable_probe_bytes: u64,
    pub full_downloads_never_useful: u64,
    pub(super) request_count: u64,
    pub(super) playable_videos: u64,
    pub(super) requests_per_playable_milli: u64,
    pub(super) connection_restarts_avoided_by_promotion: u64,
    pub(super) cpu_micros: u64,
    pub(crate) storage_byte_ms: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BudgetMetrics {
    pub(super) instantaneous_violations: u64,
    pub observations: u64,
    pub long_run_network_target_error_bps: i32,
    pub(super) long_run_storage_target_error_bps: i32,
    pub(super) shadow_price_stability_bps: u16,
    pub(super) qoe_per_matched_network_micros: u64,
    pub(super) qoe_per_matched_storage_micros: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReadinessMetrics {
    pub(super) reserve_underflows: u64,
    pub(super) reserve_underflow_ms: u64,
    pub(super) observed_ms: u64,
    pub(super) reserve_underflow_frequency_bps: u16,
    pub(super) probability_weighted_ready_reserve_millis: u64,
    pub(super) useful_ready_coverage_ms: u64,
    pub(crate) on_time_readiness_samples: u64,
    pub(crate) on_time_readiness_expected_bps: u16,
    pub(crate) on_time_readiness_observed_bps: u16,
    pub(crate) on_time_readiness_calibration_error_bps: u16,
    pub(super) on_time_readiness_calibration_bps: u16,
    pub(super) replenish_after_burst: LatencyDistribution,
    pub(super) protected_rescue_slot_claims: u64,
    pub(super) protected_rescue_slot_uses: u64,
    pub(super) protected_rescue_slot_utilization_bps: u16,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdaptationMetrics {
    pub(crate) origin_change_points: u64,
    pub(super) regret_micros: u64,
    pub(crate) recovery_after_change: LatencyDistribution,
    pub(crate) success_predictions: u64,
    pub(super) success_expected_bps: u16,
    pub(crate) success_observed_bps: u16,
    pub(crate) success_calibration_error_bps: u16,
    pub(super) latency_predictions: u64,
    pub(crate) latency_p50_coverage_bps: u16,
    pub(crate) latency_p95_coverage_bps: u16,
    pub(crate) latency_p99_coverage_bps: u16,
    pub(super) quantile_predictions: u64,
    pub(super) quantile_coverage_bps: u16,
    pub(super) exploration_bytes: u64,
    pub(super) failed_exploration_bytes: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticsMetrics {
    pub(super) focus_sessions: u64,
    pub(super) rank_displacement: u64,
    pub(super) semantic_regret_micros: u64,
    pub(super) transport_substitutions: u64,
    pub(super) transport_substitution_rate_bps: u16,
    pub(crate) exposure_by_origin: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrityMetrics {
    pub(super) hash_mismatches: u64,
    pub(super) stale_validator_incidents: u64,
    pub(super) false_streamability_classifications: u64,
    pub(super) metadata_field_calibration_errors: u64,
    pub(super) incorrect_range_splices_prevented: u64,
    pub(super) parser_limit_rejections: u64,
    pub(super) ssrf_redirect_blocks: u64,
}
