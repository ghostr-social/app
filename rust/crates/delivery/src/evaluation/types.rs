use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvaluationSnapshot {
    pub user_visible: UserVisibleMetrics,
    pub efficiency: EfficiencyMetrics,
    pub budget: BudgetMetrics,
    pub readiness: ReadinessMetrics,
    pub adaptation: AdaptationMetrics,
    pub semantics: SemanticsMetrics,
    pub integrity: IntegrityMetrics,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LatencyDistribution {
    pub samples: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserVisibleMetrics {
    pub swipe_to_first_frame: LatencyDistribution,
    pub startup_sessions: u64,
    pub startup_failures: u64,
    pub startup_failure_rate_bps: u16,
    pub stall_events: u64,
    pub stall_ms: u64,
    pub stall_ratio_bps: u16,
    pub first_frame_quality_bps: u64,
    pub watch_weighted_quality_bps: u64,
    pub quality_discontinuities: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub total_bytes: u64,
    pub useful_watched_bytes: u64,
    pub aborted_bytes: u64,
    pub duplicate_hedge_bytes: u64,
    pub completable_probe_bytes: u64,
    pub full_downloads_never_useful: u64,
    pub request_count: u64,
    pub playable_videos: u64,
    pub requests_per_playable_milli: u64,
    pub connection_restarts_avoided_by_promotion: u64,
    pub cpu_micros: u64,
    pub storage_byte_ms: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BudgetMetrics {
    pub instantaneous_violations: u64,
    pub observations: u64,
    pub long_run_network_target_error_bps: i32,
    pub long_run_storage_target_error_bps: i32,
    pub shadow_price_stability_bps: u16,
    pub qoe_per_matched_network_micros: u64,
    pub qoe_per_matched_storage_micros: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReadinessMetrics {
    pub reserve_underflows: u64,
    pub reserve_underflow_ms: u64,
    pub observed_ms: u64,
    pub reserve_underflow_frequency_bps: u16,
    pub probability_weighted_ready_reserve_millis: u64,
    pub useful_ready_coverage_ms: u64,
    pub on_time_readiness_samples: u64,
    pub on_time_readiness_expected_bps: u16,
    pub on_time_readiness_observed_bps: u16,
    pub on_time_readiness_calibration_error_bps: u16,
    pub on_time_readiness_calibration_bps: u16,
    pub replenish_after_burst: LatencyDistribution,
    pub protected_rescue_slot_claims: u64,
    pub protected_rescue_slot_uses: u64,
    pub protected_rescue_slot_utilization_bps: u16,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdaptationMetrics {
    pub origin_change_points: u64,
    pub regret_micros: u64,
    pub recovery_after_change: LatencyDistribution,
    pub success_predictions: u64,
    pub success_expected_bps: u16,
    pub success_observed_bps: u16,
    pub success_calibration_error_bps: u16,
    pub latency_predictions: u64,
    pub latency_p50_coverage_bps: u16,
    pub latency_p95_coverage_bps: u16,
    pub latency_p99_coverage_bps: u16,
    pub quantile_predictions: u64,
    pub quantile_coverage_bps: u16,
    pub exploration_bytes: u64,
    pub failed_exploration_bytes: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticsMetrics {
    pub focus_sessions: u64,
    pub rank_displacement: u64,
    pub semantic_regret_micros: u64,
    pub transport_substitutions: u64,
    pub transport_substitution_rate_bps: u16,
    pub exposure_by_origin: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrityMetrics {
    pub hash_mismatches: u64,
    pub stale_validator_incidents: u64,
    pub false_streamability_classifications: u64,
    pub metadata_field_calibration_errors: u64,
    pub incorrect_range_splices_prevented: u64,
    pub parser_limit_rejections: u64,
    pub ssrf_redirect_blocks: u64,
}
