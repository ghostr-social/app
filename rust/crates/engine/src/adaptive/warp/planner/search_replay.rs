use super::least_risk;
use crate::adaptive::{
    ActionNode, BeamConfig, HardBudget, ResourcePrices, SearchDecision, SearchPruneReason,
    WarpSearch,
};

use super::super::search::ScoredSearchPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SearchReplayMode {
    Beam,
    GreedyExpansion,
    GreedyLatency,
    LeastRisk,
}

impl SearchReplayMode {
    pub(super) fn capture(decision: &SearchDecision, degraded: bool) -> Self {
        if degraded {
            return Self::LeastRisk;
        }
        if !decision.used_greedy_fallback {
            return Self::Beam;
        }
        match decision.pruned_plans.first().map(|plan| plan.reason) {
            Some(SearchPruneReason::PlannerLatency) => Self::GreedyLatency,
            _ => Self::GreedyExpansion,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SearchReplayInput {
    pub(crate) mode: SearchReplayMode,
    pub(crate) nodes: Vec<ActionNode>,
    pub(crate) budget: HardBudget,
    pub(crate) beam: BeamConfig,
    pub(crate) prices: ResourcePrices,
    pub(crate) scores: Vec<ScoredSearchPlan>,
}

impl SearchReplayInput {
    pub(crate) const fn mode(&self) -> SearchReplayMode {
        self.mode
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

    pub(crate) fn run(&self) -> Option<SearchDecision> {
        match self.mode {
            SearchReplayMode::Beam => self.run_beam(),
            SearchReplayMode::GreedyExpansion => {
                Some(self.run_greedy(SearchPruneReason::ExpansionLimit))
            }
            SearchReplayMode::GreedyLatency => {
                Some(self.run_greedy(SearchPruneReason::PlannerLatency))
            }
            SearchReplayMode::LeastRisk => Some(least_risk::choose(&self.nodes)),
        }
    }

    fn run_beam(&self) -> Option<SearchDecision> {
        let search = WarpSearch::new(self.beam.without_latency_limit()).with_prices(self.prices);
        let mut cursor = ScoreCursor::new(&self.scores);
        let mut scorer = |actions: &[ActionNode]| cursor.score(actions);
        let decision = search.choose_first_recorded(&self.nodes, self.budget.clone(), &mut scorer);
        cursor.complete().then_some(decision)
    }

    fn run_greedy(&self, reason: SearchPruneReason) -> SearchDecision {
        WarpSearch::new(self.beam)
            .with_prices(self.prices)
            .choose_first_greedy(&self.nodes, self.budget.clone(), reason)
    }
}

struct ScoreCursor<'a> {
    expected: &'a [ScoredSearchPlan],
    next: usize,
    mismatch: bool,
}

impl<'a> ScoreCursor<'a> {
    const fn new(expected: &'a [ScoredSearchPlan]) -> Self {
        Self {
            expected,
            next: 0,
            mismatch: false,
        }
    }

    fn score(&mut self, actions: &[ActionNode]) -> i64 {
        let Some(expected) = self.expected.get(self.next) else {
            self.mismatch = true;
            return 0;
        };
        self.next += 1;
        let ids: Vec<_> = actions.iter().map(|action| action.id).collect();
        self.mismatch |= ids != expected.action_ids;
        expected.score_micros
    }

    const fn complete(&self) -> bool {
        !self.mismatch && self.next == self.expected.len()
    }
}
