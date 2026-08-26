use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

#[derive(Clone, Debug)]
pub struct PlaybackMetricEvent {
    pub(crate) post: PostId,
    pub(crate) phase: PlaybackPhase,
    pub(crate) bitrate_bps: u64,
    pub(crate) observed_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct PresentationMetricEvent {
    pub(crate) post: PostId,
    pub(crate) bitrate_bps: u64,
    pub(crate) origin: String,
    pub(crate) observed_at_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub struct TransferMetricEvent {
    pub(crate) post: Option<PostId>,
    pub(crate) total_bytes: u64,
    pub(crate) aborted_bytes: u64,
    pub(crate) duplicate_hedge_bytes: u64,
    pub(crate) completable_probe_bytes: u64,
    pub(crate) full_download_started: bool,
    pub(crate) request_started: bool,
    pub(crate) promotion_avoided_restart: bool,
    pub(crate) cpu_micros: u64,
    pub(crate) storage_byte_ms: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BudgetMetricEvent {
    pub(crate) observed_at_ms: u64,
    pub(crate) stored_bytes: u64,
    pub(crate) instantaneous_violation: bool,
    pub(crate) network_target_error_bps: i32,
    pub(crate) storage_target_error_bps: i32,
    pub(crate) shadow_price_total_micros: u64,
    pub(crate) qoe_micros: u64,
    pub(crate) matched_network_bytes: u64,
    pub(crate) matched_storage_byte_ms: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReadinessMetricEvent {
    pub(crate) observed_at_ms: u64,
    pub(crate) observed_ms: u64,
    pub(crate) underflow: bool,
    pub(crate) underflow_ms: u64,
    pub(crate) probability_weighted_reserve_millis: u64,
    pub(crate) ready_coverage_ms: u64,
    pub(crate) on_time_prediction_bps: Option<u16>,
    pub(crate) on_time_observed: Option<bool>,
    pub(crate) replenished_after_ms: Option<u64>,
    pub(crate) protected_slot_claimed: bool,
    pub(crate) protected_slot_used: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AdaptationMetricEvent {
    pub(crate) origin: String,
    pub(crate) observed_at_ms: u64,
    pub(crate) adapting: bool,
    pub(crate) predicted_success_bps: u16,
    pub(crate) succeeded: Option<bool>,
    pub(crate) latency_quantiles_on_time: Option<[bool; 3]>,
    pub(crate) regret_micros: u64,
    pub(crate) exploration_bytes: u64,
    pub(crate) failed_exploration_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SemanticMetricEvent {
    pub(crate) rank_displacement: u32,
    pub(crate) semantic_regret_micros: u64,
    pub(crate) transport_substitution: bool,
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
