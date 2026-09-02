use super::advanced::{self, RecordedResourceCost, RecordedWarpDecision};
use super::model;
use super::plan;
use super::privacy::DecisionPrivacy;
use super::state::ReplayState;
use super::types::{
    DecisionAction, DecisionModelInput, DecisionOutcome, ModelQuantiles, PrunedCandidate,
    ShadowPrices,
};
use crate::adaptive::{AllocationPlan, PlayabilitySnapshot, WarpPlanningDecision};
use serde::{Deserialize, Serialize};

mod authorization;
mod binding;
mod resolution;

pub(super) const UNSEALED_WARP_SCHEMA_VERSION: u16 = 2;
pub(super) const WARP_SCHEMA_VERSION: u16 = 3;
pub(super) const CAPABILITY_SCHEMA_VERSION: u16 = 4;
const ORDERED_RESERVE_SCHEMA_VERSION: u16 = 5;
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
    admissible_candidates: Vec<String>,
    retained_plans: Vec<DecisionAction>,
    pruned: Vec<PrunedCandidate>,
    model_quantiles: Vec<ModelQuantiles>,
    shadow_prices: ShadowPrices,
    pub chosen_action: Option<DecisionAction>,
    pub chosen_action_id: Option<u64>,
    pub eventual_outcome: DecisionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_resources: Option<RecordedResourceCost>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warp_decision: Option<RecordedWarpDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_request: Option<advanced::RecordedExecutedRequest>,
    replay_state: ReplayState,
}

impl DecisionRecord {
    pub fn capture(input: DecisionRecordInput<'_>) -> Self {
        let replay_state = ReplayState::capture(input.snapshot, input.privacy);
        policy_record(input, replay_state)
    }

    /// Captures the selected WARP command. A selected record must later be bound or resolved.
    #[must_use]
    pub fn capture_warp(input: WarpDecisionRecordInput<'_>) -> Self {
        let replay_state = ReplayState::capture(input.snapshot, input.privacy);
        let captured = advanced::capture(input.decision, input.privacy);
        warp_record(input, replay_state, captured)
    }
}

fn policy_record(input: DecisionRecordInput<'_>, replay_state: ReplayState) -> DecisionRecord {
    let retained_plans = plan::actions(input.allocation, input.privacy);
    DecisionRecord {
        schema_version: ORDERED_RESERVE_SCHEMA_VERSION,
        sequence: input.sequence,
        admissible_candidates: plan::admissible(input.snapshot, input.privacy),
        chosen_action: retained_plans.iter().find(|item| !item.retained).cloned(),
        chosen_action_id: None,
        retained_plans,
        pruned: plan::pruned(input.snapshot, input.allocation, input.privacy),
        model_quantiles: model::capture(input.models, input.privacy),
        shadow_prices: input.shadow_prices,
        eventual_outcome: DecisionOutcome::Pending,
        actual_resources: None,
        warp_decision: None,
        executed_request: None,
        replay_state,
    }
}

fn warp_record(
    input: WarpDecisionRecordInput<'_>,
    replay_state: ReplayState,
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
        admissible_candidates: captured.admissible_candidates,
        retained_plans: Vec::new(),
        pruned: Vec::new(),
        model_quantiles: model::capture(input.models, input.privacy),
        shadow_prices: input.legacy_shadow_prices,
        chosen_action: captured.chosen_action,
        chosen_action_id: None,
        eventual_outcome: outcome,
        actual_resources: None,
        warp_decision: Some(captured.decision),
        executed_request: None,
        replay_state,
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
