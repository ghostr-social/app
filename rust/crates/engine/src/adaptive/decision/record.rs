use super::advanced::{self, RecordedResourceCost, RecordedWarpDecision};
use super::model;
use super::plan;
use super::plan_identity;
use super::privacy::DecisionPrivacy;
use super::state::ReplayState;
use super::types::{
    DecisionAction, DecisionModelInput, DecisionOutcome, DecisionReplayStatus, ModelQuantiles,
    PrunedCandidate, ShadowPrices,
};
use crate::adaptive::{AllocationPlan, PlayabilitySnapshot, WarpPlanningDecision};
use serde::{Deserialize, Serialize};

mod authorization;
mod binding;
mod replay;
#[cfg(test)]
mod replay_api_test;
mod resolution;

pub(super) const LEGACY_SCHEMA_VERSION: u16 = 1;
pub(super) const UNSEALED_WARP_SCHEMA_VERSION: u16 = 2;
pub(super) const WARP_SCHEMA_VERSION: u16 = 3;
pub(super) const CAPABILITY_SCHEMA_VERSION: u16 = 4;
pub(super) const ORDERED_RESERVE_SCHEMA_VERSION: u16 = 5;
#[derive(Clone, Copy)]
pub struct DecisionRecordInput<'a> {
    pub sequence: u64,
    pub snapshot: &'a PlayabilitySnapshot,
    pub allocation: &'a AllocationPlan,
    pub shadow_prices: ShadowPrices,
    pub models: &'a [DecisionModelInput],
    pub privacy: &'a DecisionPrivacy,
}

#[derive(Clone, Copy)]
pub struct WarpDecisionRecordInput<'a> {
    pub sequence: u64,
    pub snapshot: &'a PlayabilitySnapshot,
    pub decision: &'a WarpPlanningDecision,
    pub legacy_shadow_prices: ShadowPrices,
    pub models: &'a [DecisionModelInput],
    pub privacy: &'a DecisionPrivacy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub schema_version: u16,
    pub sequence: u64,
    pub(crate) state_hash: String,
    pub(crate) admissible_candidates: Vec<String>,
    pub(crate) retained_plans: Vec<DecisionAction>,
    pub(crate) pruned: Vec<PrunedCandidate>,
    pub(crate) model_quantiles: Vec<ModelQuantiles>,
    pub(crate) shadow_prices: ShadowPrices,
    pub chosen_action: Option<DecisionAction>,
    pub chosen_action_id: Option<u64>,
    pub(crate) random_seed: u64,
    pub eventual_outcome: DecisionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_resources: Option<RecordedResourceCost>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warp_decision: Option<RecordedWarpDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_request: Option<advanced::RecordedExecutedRequest>,
    replay_state: ReplayState,
    replay_plan_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    terminal_evidence_hash: Option<String>,
}

impl DecisionRecord {
    pub fn capture(input: DecisionRecordInput<'_>) -> Self {
        let replay_state = ReplayState::capture(input.snapshot, input.privacy);
        let (state_hash, random_seed) = replay::state_identity(&replay_state);
        policy_record(input, replay_state, state_hash, random_seed)
    }

    /// Captures the selected WARP command. A selected record must later be bound or resolved.
    #[must_use]
    pub fn capture_warp(input: WarpDecisionRecordInput<'_>) -> Self {
        let replay_state = ReplayState::capture(input.snapshot, input.privacy);
        let state_hash = replay::warp_state_identity(&replay_state).0;
        let captured = advanced::capture(input.decision, input.privacy);
        let mut record = warp_record(input, replay_state, state_hash, captured);
        record.replay_plan_hash = replay::warp_identity(&record);
        record.terminal_evidence_hash = Some(replay::terminal_identity(&record));
        record
    }

    /// Verifies the retained decision's privacy-safe replay envelope and search trace.
    #[must_use]
    pub fn integrity_status(&self) -> DecisionReplayStatus {
        replay::status(self)
    }

    /// Re-executes the captured WARP search and verifies its exact output.
    #[must_use]
    pub fn search_integrity_status(&self) -> DecisionReplayStatus {
        replay::search_status(self)
    }
}

fn policy_record(
    input: DecisionRecordInput<'_>,
    replay_state: ReplayState,
    state_hash: String,
    random_seed: u64,
) -> DecisionRecord {
    let retained_plans = plan::actions(input.allocation, input.privacy);
    DecisionRecord {
        schema_version: ORDERED_RESERVE_SCHEMA_VERSION,
        sequence: input.sequence,
        state_hash,
        admissible_candidates: plan::admissible(input.snapshot, input.privacy),
        chosen_action: retained_plans.iter().find(|item| !item.retained).cloned(),
        chosen_action_id: None,
        retained_plans,
        pruned: plan::pruned(input.snapshot, input.allocation, input.privacy),
        model_quantiles: model::capture(input.models, input.privacy),
        shadow_prices: input.shadow_prices,
        random_seed,
        eventual_outcome: DecisionOutcome::Pending,
        actual_resources: None,
        warp_decision: None,
        executed_request: None,
        replay_state,
        replay_plan_hash: plan_identity::capture_ordered(input.allocation, input.privacy),
        terminal_evidence_hash: None,
    }
}

fn warp_record(
    input: WarpDecisionRecordInput<'_>,
    replay_state: ReplayState,
    state_hash: String,
    captured: advanced::WarpCapture,
) -> DecisionRecord {
    let outcome = initial_warp_outcome(captured.decision.selected.is_some());
    let schema_version = if replay_state.has_direct_playback_block() {
        CAPABILITY_SCHEMA_VERSION
    } else {
        WARP_SCHEMA_VERSION
    };
    DecisionRecord {
        schema_version,
        sequence: input.sequence,
        state_hash,
        admissible_candidates: captured.admissible_candidates,
        retained_plans: Vec::new(),
        pruned: Vec::new(),
        model_quantiles: model::capture(input.models, input.privacy),
        shadow_prices: input.legacy_shadow_prices,
        chosen_action: captured.chosen_action,
        chosen_action_id: None,
        random_seed: captured.random_seed,
        eventual_outcome: outcome,
        actual_resources: None,
        warp_decision: Some(captured.decision),
        executed_request: None,
        replay_state,
        replay_plan_hash: String::new(),
        terminal_evidence_hash: None,
    }
}

fn initial_warp_outcome(selected: bool) -> DecisionOutcome {
    if selected {
        DecisionOutcome::Pending
    } else {
        DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 0,
        }
    }
}
