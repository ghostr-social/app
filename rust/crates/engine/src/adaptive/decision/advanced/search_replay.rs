use super::{
    RecordedReserveDegradedReason, RecordedResourceCost, RecordedResourcePrices,
    RecordedWarpActionKind, RecordedWarpReserve,
};
use serde::{Deserialize, Serialize};

mod capture;

pub(super) use capture::capture;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpSearchInput {
    mode: RecordedSearchReplayMode,
    beam: RecordedBeamConfig,
    prices: RecordedResourcePrices,
    budget: RecordedSearchBudget,
    actions: Vec<RecordedSearchAction>,
    scores: Vec<RecordedSearchScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reserve: Option<RecordedWarpReserve>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reserve_threshold_bps: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reserve_degraded_reason: Option<RecordedReserveDegradedReason>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    reserve_progress_action_ids: Vec<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecordedSearchReplayMode {
    Core,
    Beam,
    GreedyExpansion,
    GreedyLatency,
    LeastRisk,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedBeamConfig {
    depth: u64,
    width: u64,
    max_expansions: u64,
    max_latency_us: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedSearchBudget {
    remaining: RecordedResourceCost,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    segmented_storage_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    global_request_width: Option<u16>,
    per_origin_requests: u64,
    origins: Vec<RecordedAuthorityOccupancy>,
    pending_rescue_action_ids: Vec<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedAuthorityOccupancy {
    source_id: String,
    requests: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedSearchAction {
    planner_action_id: u16,
    post_id: String,
    kind: RecordedWarpActionKind,
    value: RecordedActionValue,
    resources: RecordedResourceCost,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authorized_resources: Option<RecordedResourceCost>,
    #[serde(
        default,
        skip_serializing_if = "super::RecordedOriginAdmissionIntent::is_delivery"
    )]
    origin_admission_intent: super::RecordedOriginAdmissionIntent,
    forecast: RecordedActionForecast,
    request_source_id: Option<String>,
    dependencies: Vec<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedActionValue {
    delay_loss_micros: i64,
    reserve_gain_micros: i64,
    information_value_micros: i64,
    exploration_micros: i64,
    cache_gain_micros: i64,
    tail_risk_micros: i64,
    cvar_micros: i64,
    rank_cost_micros: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedActionForecast {
    completion: RecordedCompletionTimes,
    success_bps: u16,
    ready_playback_ms: u64,
    quality_gain_micros: u64,
    cache_reuse_bps: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedCompletionTimes {
    expected_ms: u64,
    p95_ms: u64,
    p99_ms: u64,
    cvar_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedSearchScore {
    action_ids: Vec<u16>,
    score_micros: i64,
}
