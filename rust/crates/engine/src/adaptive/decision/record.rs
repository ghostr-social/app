use super::advanced::{self, RecordedResourceCost, RecordedWarpDecision};
use super::model;
use super::plan;
use super::privacy::DecisionPrivacy;
use super::state::ReplayState;
use super::types::{
    DecisionAction, DecisionModelInput, DecisionOutcome, DecisionReplayStatus, ModelQuantiles,
    PrunedCandidate, ShadowPrices,
};
use crate::adaptive::VerifiedWarpReplay;
use crate::adaptive::{AllocationPlan, PlayabilitySnapshot, WarpPlanningDecision};
use serde::{Deserialize, Serialize};

mod authorization;
mod binding;
mod replay;
mod resolution;

pub(super) const LEGACY_SCHEMA_VERSION: u16 = 1;
pub(super) const UNSEALED_WARP_SCHEMA_VERSION: u16 = 2;
pub(super) const WARP_SCHEMA_VERSION: u16 = 3;
pub struct DecisionRecordInput<'a> {
    pub sequence: u64,
    pub snapshot: &'a PlayabilitySnapshot,
    pub allocation: &'a AllocationPlan,
    pub shadow_prices: ShadowPrices,
    pub models: &'a [DecisionModelInput],
    pub privacy: &'a DecisionPrivacy,
}

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
    pub state_hash: String,
    pub admissible_candidates: Vec<String>,
    pub retained_plans: Vec<DecisionAction>,
    pub pruned: Vec<PrunedCandidate>,
    pub model_quantiles: Vec<ModelQuantiles>,
    pub shadow_prices: ShadowPrices,
    pub chosen_action: Option<DecisionAction>,
    pub chosen_action_id: Option<u64>,
    pub random_seed: u64,
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
        legacy_record(input, replay_state, state_hash, random_seed)
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

    pub fn replay(&self) -> DecisionReplayStatus {
        replay::status(self)
    }

    pub fn replay_warp(&self) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
        replay::warp(self)
    }

    /// Re-executes the captured privacy-safe WARP search and verifies its exact output.
    pub fn replay_warp_search(&self) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
        replay::warp_search(self)
    }
}

fn legacy_record(
    input: DecisionRecordInput<'_>,
    replay_state: ReplayState,
    state_hash: String,
    random_seed: u64,
) -> DecisionRecord {
    let retained_plans = plan::actions(input.allocation, input.privacy);
    DecisionRecord {
        schema_version: LEGACY_SCHEMA_VERSION,
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
        replay_plan_hash: plan::capture_hash(input.allocation, input.privacy),
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
    DecisionRecord {
        schema_version: WARP_SCHEMA_VERSION,
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
    match selected {
        true => DecisionOutcome::Pending,
        false => DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 0,
        },
    }
}
