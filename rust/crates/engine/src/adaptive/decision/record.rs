use super::model;
use super::plan;
use super::privacy::DecisionPrivacy;
use super::state::ReplayState;
use super::types::{
    DecisionAction, DecisionModelInput, DecisionOutcome, DecisionReplayStatus, ModelQuantiles,
    PrunedCandidate, ShadowPrices,
};
use crate::adaptive::{AdaptivePlayabilityPolicy, AllocationPlan, PlayabilitySnapshot};
use crate::ActionId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u16 = 1;

pub struct DecisionRecordInput<'a> {
    pub sequence: u64,
    pub snapshot: &'a PlayabilitySnapshot,
    pub allocation: &'a AllocationPlan,
    pub shadow_prices: ShadowPrices,
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
    replay_state: ReplayState,
    replay_plan_hash: String,
}

impl DecisionRecord {
    pub fn capture(input: DecisionRecordInput<'_>) -> Self {
        let DecisionRecordInput {
            sequence,
            snapshot,
            allocation,
            shadow_prices,
            models,
            privacy,
        } = input;
        let replay_state = ReplayState::capture(snapshot, privacy);
        let (state_hash, random_seed) = state_identity(&replay_state);
        let retained_plans = plan::actions(allocation, privacy);
        Self {
            schema_version: SCHEMA_VERSION,
            sequence,
            state_hash,
            admissible_candidates: plan::admissible(snapshot, privacy),
            chosen_action: retained_plans.iter().find(|item| !item.retained).cloned(),
            chosen_action_id: None,
            retained_plans,
            pruned: plan::pruned(snapshot, allocation, privacy),
            model_quantiles: model::capture(models, privacy),
            shadow_prices,
            random_seed,
            eventual_outcome: DecisionOutcome::Pending,
            replay_state,
            replay_plan_hash: plan::capture_hash(allocation, privacy),
        }
    }

    pub fn replay(&self) -> DecisionReplayStatus {
        if state_identity(&self.replay_state).0 != self.state_hash {
            return DecisionReplayStatus::StateHashMismatch;
        }
        let snapshot = self.replay_state.snapshot();
        let replayed = AdaptivePlayabilityPolicy.plan(&snapshot);
        match plan::replay_hash(&replayed) == self.replay_plan_hash {
            true => DecisionReplayStatus::Verified,
            false => DecisionReplayStatus::PlanMismatch,
        }
    }

    pub fn resolve(&mut self, outcome: DecisionOutcome) -> bool {
        if self.eventual_outcome != DecisionOutcome::Pending {
            return false;
        }
        self.eventual_outcome = outcome;
        true
    }

    pub fn bind_action(&mut self, action: ActionId) -> bool {
        if self.chosen_action.is_none() || self.chosen_action_id.is_some() {
            return false;
        }
        self.chosen_action_id = Some(action.value());
        true
    }
}

fn state_identity(state: &ReplayState) -> (String, u64) {
    let encoded = serde_json::to_vec(state).expect("replay state is serializable");
    let digest = Sha256::digest(encoded);
    let mut seed = [0; 8];
    seed.copy_from_slice(&digest[..8]);
    let seed = u64::from_be_bytes(seed).max(1);
    (hex(&digest), seed)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
