use crate::adaptive::{
    ActionNode, BeamConfig, HardBudget, ReserveConstraint, ReserveDegradedReason, ResourcePrices,
    SearchDecision, SearchPruneReason,
};

use super::super::search::ScoredSearchPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SearchReplayMode {
    Core,
    Beam,
    GreedyExpansion,
    GreedyLatency,
    LeastRisk,
}

impl SearchReplayMode {
    pub(super) fn capture(
        decision: &SearchDecision,
        degraded: bool,
        reserve_progress: bool,
    ) -> Self {
        let reason = decision.pruned_plans.first().map(|plan| plan.reason);
        if degraded || reserve_progress || reason == Some(SearchPruneReason::DemandedInput) {
            return Self::LeastRisk;
        }
        if !decision.used_greedy_fallback {
            return Self::Beam;
        }
        match reason {
            Some(SearchPruneReason::OptimizationDisabled) => Self::Core,
            Some(SearchPruneReason::PlannerLatency) => Self::GreedyLatency,
            _ => Self::GreedyExpansion,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SearchReplayInput {
    pub(super) mode: SearchReplayMode,
    pub(super) reserve: ReserveConstraint,
    pub(super) reserve_threshold_bps: Option<u16>,
    pub(super) reserve_degraded_reason: Option<ReserveDegradedReason>,
    pub(super) nodes: Vec<ActionNode>,
    pub(super) budget: HardBudget,
    pub(super) beam: BeamConfig,
    pub(super) prices: ResourcePrices,
    pub(super) scores: Vec<ScoredSearchPlan>,
    pub(super) reserve_progress_action_ids: Vec<u16>,
}

impl SearchReplayInput {
    pub(crate) const fn mode(&self) -> SearchReplayMode {
        self.mode
    }

    pub(crate) const fn reserve(&self) -> &ReserveConstraint {
        &self.reserve
    }

    pub(crate) const fn reserve_threshold_bps(&self) -> Option<u16> {
        self.reserve_threshold_bps
    }

    pub(crate) const fn reserve_degraded_reason(&self) -> Option<ReserveDegradedReason> {
        self.reserve_degraded_reason
    }

    pub(crate) fn nodes(&self) -> &[ActionNode] {
        &self.nodes
    }

    pub(crate) const fn budget(&self) -> &HardBudget {
        &self.budget
    }

    pub(crate) const fn beam(&self) -> BeamConfig {
        self.beam
    }

    pub(crate) const fn prices(&self) -> ResourcePrices {
        self.prices
    }

    pub(crate) fn scores(&self) -> &[ScoredSearchPlan] {
        &self.scores
    }

    pub(crate) fn reserve_progress_action_ids(&self) -> &[u16] {
        &self.reserve_progress_action_ids
    }
}
