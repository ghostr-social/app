//! Privacy-safe, selection-only WARP decision projection for schema v2.

mod capture;
mod command;
mod executed;
mod hls;
mod kind;
mod origin_intent;
mod planner_replay;
mod reserve;
mod search;
mod search_replay;

pub use command::{
    RecordedAllocationReason, RecordedCandidateUtility, RecordedPreemptionAuthority,
    RecordedPromotionGrant, RecordedRetrievalRequest, RecordedTransfer, RecordedTransformKind,
    RecordedWarpCommand, RecordedWholeBodyContract, RecordedWholeFetchReason,
};
pub use executed::RecordedExecutedRequest;
pub(in crate::adaptive::decision) use executed::{
    capture as capture_executed, coherent as executed_coherent,
};
pub use hls::RecordedHlsBootstrapStage;
pub use kind::RecordedWarpActionKind;
use origin_intent::RecordedOriginAdmissionIntent;
pub use planner_replay::RecordedPlannerReplayCapsule;
pub use reserve::{
    RecordedRescueChanceEvidence, RecordedRescueTimingQuantile, RecordedReserveAuthorityOccupancy,
    RecordedReserveDegradedReason, RecordedWarpReserve,
};
pub use search_replay::RecordedWarpSearchInput;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpDecision {
    pub selected: Option<RecordedWarpAction>,
    admissible_actions: Vec<RecordedWarpAction>,
    admissible_actions_total: u64,
    unattributed_pre_search_pruned_actions: Vec<RecordedWarpAction>,
    unattributed_pre_search_pruned_actions_total: u64,
    pub search: RecordedWarpSearch,
    pub prices: RecordedResourcePrices,
    evaluation: Option<RecordedTwinEvaluation>,
    reserve: RecordedWarpReserve,
    additional_request_slot_demanded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    retry_availability: Vec<RecordedPlannerRetryEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    search_replay_input: Option<RecordedWarpSearchInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    planner_replay_capsule: Option<RecordedPlannerReplayCapsule>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedPlannerRetryEvidence {
    post_id: String,
    availability: RecordedPlannerRetryAvailability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedPlannerRetryAvailability {
    Ready,
    Cooling { eligible_at_ms: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpSearch {
    committed_actions: u8,
    used_greedy_fallback: bool,
    chosen_plan: Option<RecordedRetainedSearchPlan>,
    retained_plans: Vec<RecordedRetainedSearchPlan>,
    retained_plans_total: u64,
    /// Search already keeps only a bounded audit sample; these are those recorded prunes.
    recorded_pruned_plans: Vec<RecordedPrunedSearchPlan>,
    pruned_plan_events_total: u64,
    pruned_plan_sample_truncated: bool,
    recorder_truncated_pruned_plans: bool,
    pub common_random_seed: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedRetainedSearchPlan {
    actions: Vec<RecordedWarpAction>,
    actions_total: u64,
    score_micros: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedPrunedSearchPlan {
    actions: Vec<RecordedWarpAction>,
    actions_total: u64,
    reason: RecordedSearchPruneReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedSearchPruneReason {
    OptimizationDisabled,
    DemandedInput,
    HardBudget,
    MutuallyExclusive,
    BeamWidth,
    ExpansionLimit,
    PlannerLatency,
    ReserveUnderflow,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpAction {
    planner_action_id: u16,
    post_id: String,
    kind: RecordedWarpActionKind,
    pub command: RecordedWarpCommand,
    pub resources: RecordedResourceCost,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authorized_resources: Option<RecordedResourceCost>,
    #[serde(
        default,
        skip_serializing_if = "RecordedOriginAdmissionIntent::is_delivery"
    )]
    origin_admission_intent: RecordedOriginAdmissionIntent,
    dependencies: Vec<u16>,
    ready_playback_ms: u64,
    static_score_micros: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedResourceCost {
    pub network_bytes: u64,
    pub storage_bytes: u64,
    pub cpu_ms: u64,
    pub requests: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedResourcePrices {
    network_micros: u64,
    storage_micros: u64,
    pub cpu_micros: u64,
    request_micros: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedTwinEvaluation {
    expected_score_micros: i64,
    expected_visible_delay_ms: u64,
    p95_visible_delay_ms: u64,
    p99_visible_delay_ms: u64,
    cvar_visible_delay_ms: u64,
    on_time_probability_bps: u16,
    expected_ready_coverage_ms: u64,
    expected_cache_bytes: u64,
    common_random_seed: u64,
}

pub(super) use capture::{capture, WarpCapture};
