use super::super::{
    BeamConfig, GeneratedAction, GeneratedActions, PlannerContext, PlannerRetryEvidence,
    ResourcePrices, SearchDecision, SemanticAdmission, TwinConfig, TwinEvaluation,
};
use super::reserve::ReserveConstraint;
use super::{PlannerReplayCapsule, SearchReplayInput};
use crate::adaptive::{AllocationPlan, PlayabilitySnapshot};
use crate::origin_model::OriginModel;
use crate::PostId;

#[cfg(test)]
mod api_test;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WarpPlannerConfig {
    pub(crate) beam: BeamConfig,
    pub(crate) twin: TwinConfig,
    pub(crate) semantic_top_k: usize,
    pub(crate) semantic_epsilon_micros: u64,
    pub(crate) safety_rescue_bps: u16,
    pub(crate) emergency_rescue_bps: u16,
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

#[derive(Clone, Copy)]
pub struct WarpPlannerInput<'a> {
    pub(super) snapshot: &'a PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) origins: &'a OriginModel,
    pub(super) context: &'a PlannerContext,
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
    pub(super) post: PostId,
    pub(super) admission: SemanticAdmission,
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
    pub(crate) reserve: ReserveConstraint,
    pub(crate) semantic: Vec<SemanticDecision>,
    pub prices: ResourcePrices,
    pub common_random_seed: u64,
    pub retry_availability: Vec<PlannerRetryEvidence>,
    pub(crate) search_replay: Option<SearchReplayInput>,
    pub(crate) planner_replay: Option<PlannerReplayCapsule>,
}

#[cfg(any(test, feature = "test"))]
#[path = "types/test_support.rs"]
mod test_support;
