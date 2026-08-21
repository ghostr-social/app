//! Privacy-safe, selection-only WARP decision projection for schema v2.

mod capture;
mod command;
mod kind;

pub use command::{
    RecordedAllocationReason, RecordedCandidateUtility, RecordedPreemptionAuthority,
    RecordedPromotionGrant, RecordedRetrievalRequest, RecordedTransfer, RecordedTransformKind,
    RecordedWarpCommand, RecordedWholeBodyContract, RecordedWholeFetchReason,
};
pub use kind::RecordedWarpActionKind;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpDecision {
    pub selected: Option<RecordedWarpAction>,
    pub admissible_action_ids: Vec<u16>,
    pub prices: RecordedResourcePrices,
    pub evaluation: Option<RecordedTwinEvaluation>,
    pub reserve: RecordedWarpReserve,
    pub additional_request_slot_demanded: bool,
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
pub struct RecordedWarpReserve {
    pub reserved_request_slots: u16,
    pub reserved_network_bytes: u64,
    pub degraded: bool,
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
