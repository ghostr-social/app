use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

#[derive(Clone, Debug)]
pub struct PlaybackMetricEvent {
    pub post: PostId,
    pub phase: PlaybackPhase,
    pub bitrate_bps: u64,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct PresentationMetricEvent {
    pub post: PostId,
    pub bitrate_bps: u64,
    pub origin: String,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub struct TransferMetricEvent {
    pub post: Option<PostId>,
    pub total_bytes: u64,
    pub aborted_bytes: u64,
    pub duplicate_hedge_bytes: u64,
    pub completable_probe_bytes: u64,
    pub full_download_started: bool,
    pub request_started: bool,
    pub promotion_avoided_restart: bool,
    pub cpu_micros: u64,
    pub storage_byte_ms: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BudgetMetricEvent {
    pub observed_at_ms: u64,
    pub stored_bytes: u64,
    pub instantaneous_violation: bool,
    pub network_target_error_bps: i32,
    pub storage_target_error_bps: i32,
    pub shadow_price_total_micros: u64,
    pub qoe_micros: u64,
    pub matched_network_bytes: u64,
    pub matched_storage_byte_ms: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReadinessMetricEvent {
    pub observed_at_ms: u64,
    pub observed_ms: u64,
    pub underflow: bool,
    pub underflow_ms: u64,
    pub probability_weighted_reserve_millis: u64,
    pub ready_coverage_ms: u64,
    pub on_time_prediction_bps: Option<u16>,
    pub on_time_observed: Option<bool>,
    pub replenished_after_ms: Option<u64>,
    pub protected_slot_claimed: bool,
    pub protected_slot_used: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AdaptationMetricEvent {
    pub origin: String,
    pub observed_at_ms: u64,
    pub adapting: bool,
    pub predicted_success_bps: u16,
    pub succeeded: Option<bool>,
    pub latency_quantiles_on_time: Option<[bool; 3]>,
    pub regret_micros: u64,
    pub exploration_bytes: u64,
    pub failed_exploration_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SemanticMetricEvent {
    pub rank_displacement: u32,
    pub semantic_regret_micros: u64,
    pub transport_substitution: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityMetricEvent {
    HashMismatch,
    StaleValidator,
    FalseStreamability,
    MetadataCalibrationError,
    IncorrectRangeSplicePrevented,
    ParserLimitRejection,
    SsrfOrRedirectBlock,
}
