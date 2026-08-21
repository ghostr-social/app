//! Privacy-safe, selection-only WARP decision projection for schema v2.

mod capture;
mod command;
mod executed;
mod kind;
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
pub use kind::RecordedWarpActionKind;
pub use reserve::{
    RecordedRescueChanceEvidence, RecordedRescueTimingQuantile, RecordedReserveAuthorityOccupancy,
    RecordedReserveDegradedReason, RecordedWarpReserve,
};
pub use search_replay::RecordedWarpSearchInput;
pub(in crate::adaptive::decision) use search_replay::{
    verify as verify_search_replay, verify_reserve as verify_search_reserve,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpDecision {
    pub selected: Option<RecordedWarpAction>,
    pub admissible_actions: Vec<RecordedWarpAction>,
    pub admissible_actions_total: u64,
    pub unattributed_pre_search_pruned_actions: Vec<RecordedWarpAction>,
    pub unattributed_pre_search_pruned_actions_total: u64,
    pub search: RecordedWarpSearch,
    pub prices: RecordedResourcePrices,
    pub evaluation: Option<RecordedTwinEvaluation>,
    pub reserve: RecordedWarpReserve,
    pub additional_request_slot_demanded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retry_availability: Vec<RecordedPlannerRetryEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_replay_input: Option<RecordedWarpSearchInput>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedPlannerRetryEvidence {
    pub post_id: String,
    pub availability: RecordedPlannerRetryAvailability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedPlannerRetryAvailability {
    Ready,
    Cooling { eligible_at_ms: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpSearch {
    pub committed_actions: u8,
    pub used_greedy_fallback: bool,
    pub chosen_plan: Option<RecordedRetainedSearchPlan>,
    pub retained_plans: Vec<RecordedRetainedSearchPlan>,
    pub retained_plans_total: u64,
    /// Search already keeps only a bounded audit sample; these are those recorded prunes.
    pub recorded_pruned_plans: Vec<RecordedPrunedSearchPlan>,
    pub pruned_plan_events_total: u64,
    pub pruned_plan_sample_truncated: bool,
    pub recorder_truncated_pruned_plans: bool,
    pub common_random_seed: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedRetainedSearchPlan {
    pub actions: Vec<RecordedWarpAction>,
    pub actions_total: u64,
    pub score_micros: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedPrunedSearchPlan {
    pub actions: Vec<RecordedWarpAction>,
    pub actions_total: u64,
    pub reason: RecordedSearchPruneReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedSearchPruneReason {
    HardBudget,
    MutuallyExclusive,
    BeamWidth,
    ExpansionLimit,
    PlannerLatency,
    ReserveUnderflow,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpAction {
    pub planner_action_id: u16,
    pub post_id: String,
    pub kind: RecordedWarpActionKind,
    pub command: RecordedWarpCommand,
    pub resources: RecordedResourceCost,
    pub dependencies: Vec<u16>,
    pub ready_playback_ms: u64,
    pub static_score_micros: i64,
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
    pub network_micros: u64,
    pub storage_micros: u64,
    pub cpu_micros: u64,
    pub request_micros: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedTwinEvaluation {
    pub expected_score_micros: i64,
    pub expected_visible_delay_ms: u64,
    pub p95_visible_delay_ms: u64,
    pub p99_visible_delay_ms: u64,
    pub cvar_visible_delay_ms: u64,
    pub on_time_probability_bps: u16,
    pub expected_ready_coverage_ms: u64,
    pub expected_cache_bytes: u64,
    pub common_random_seed: u64,
}

pub(super) use capture::{capture, WarpCapture};
