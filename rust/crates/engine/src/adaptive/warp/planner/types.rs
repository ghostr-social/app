use super::super::{
    BeamConfig, GeneratedAction, GeneratedActions, PlannerCandidateContext, PlannerContext,
    PlannerRetryEvidence, ResourcePrices, SearchDecision, SemanticAdmission, TwinConfig,
    TwinEpochs, TwinEvaluation,
};
use super::reserve::ReserveConstraint;
use super::{PlannerReplayCapsule, SearchReplayInput};
use crate::adaptive::{AllocationPlan, PlayabilitySnapshot};
use crate::origin_model::{NetworkClass, OriginModel};
use crate::PostId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WarpPlannerConfig {
    pub beam: BeamConfig,
    pub twin: TwinConfig,
    pub semantic_top_k: usize,
    pub semantic_epsilon_micros: u64,
    pub safety_rescue_bps: u16,
    pub emergency_rescue_bps: u16,
}

impl WarpPlannerConfig {
    pub const fn with_rescue_thresholds(mut self, safety_bps: u16, emergency_bps: u16) -> Self {
        self.safety_rescue_bps = clamp_bps(safety_bps);
        self.emergency_rescue_bps = clamp_bps(emergency_bps);
        self
    }
}

impl Default for WarpPlannerConfig {
    fn default() -> Self {
        Self {
            beam: BeamConfig::new(4, 32, 256, 2_000),
            twin: TwinConfig::new(48, 9_500),
            semantic_top_k: 5,
            semantic_epsilon_micros: 0,
            safety_rescue_bps: 9_500,
            emergency_rescue_bps: 9_900,
        }
    }
}

pub struct WarpPlannerInput<'a> {
    pub snapshot: &'a PlayabilitySnapshot,
    pub base: &'a AllocationPlan,
    pub origins: &'a OriginModel,
    pub context: &'a PlannerContext,
}

impl<'a> WarpPlannerInput<'a> {
    pub const fn new(
        snapshot: &'a PlayabilitySnapshot,
        base: &'a AllocationPlan,
        origins: &'a OriginModel,
        context: &'a PlannerContext,
    ) -> Self {
        Self {
            snapshot,
            base,
            origins,
            context,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticDecision {
    pub post: PostId,
    pub admission: SemanticAdmission,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WarpPlanningDecision {
    pub generated: GeneratedActions,
    pub selected: Option<GeneratedAction>,
    /// With one extra global slot, the planner selects a request-consuming commitment.
    pub additional_request_slot_demanded: bool,
    pub search: SearchDecision,
    pub evaluation: Option<TwinEvaluation>,
    pub admissible_action_ids: Vec<u16>,
    pub pruned_action_ids: Vec<u16>,
    pub reserve: ReserveConstraint,
    pub semantic: Vec<SemanticDecision>,
    pub prices: ResourcePrices,
    pub common_random_seed: u64,
    pub retry_availability: Vec<PlannerRetryEvidence>,
    pub(crate) search_replay: Option<SearchReplayInput>,
    pub(crate) planner_replay: Option<PlannerReplayCapsule>,
}

impl WarpPlanningDecision {
    pub fn planner_candidate_evidence(&self, post: &PostId) -> Option<PlannerCandidateContext> {
        self.planner_replay.as_ref()?.context().candidate(post)
    }

    pub fn planner_epochs(&self) -> Option<TwinEpochs> {
        Some(self.planner_replay.as_ref()?.context().epochs)
    }

    pub fn planner_network_class(&self) -> Option<NetworkClass> {
        Some(self.planner_replay.as_ref()?.context().network_class())
    }
}

const fn clamp_bps(value: u16) -> u16 {
    if value > 10_000 {
        10_000
    } else {
        value
    }
}
