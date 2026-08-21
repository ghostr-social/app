mod greedy;
mod output;
mod progress;
mod scoring;
mod state;

pub use output::{PrunedSearchPlan, RetainedSearchPlan, SearchDecision, SearchPruneReason};
pub(crate) use scoring::ScoredSearchPlan;
pub use scoring::TwinSearchContext;

use self::output::{decision, pruned_plan};
use self::progress::SearchProgress;
use self::state::{compare, State};
use super::{ActionNode, HardBudget, ResourcePrices};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BeamConfig {
    depth: usize,
    width: usize,
    max_expansions: usize,
    max_latency_us: u64,
}

impl BeamConfig {
    pub const fn new(
        depth: usize,
        width: usize,
        max_expansions: usize,
        max_latency_us: u64,
    ) -> Self {
        Self {
            depth,
            width,
            max_expansions,
            max_latency_us,
        }
    }

    pub(crate) const fn replay_parts(self) -> (usize, usize, usize, u64) {
        (
            self.depth,
            self.width,
            self.max_expansions,
            self.max_latency_us,
        )
    }

    pub(crate) const fn without_latency_limit(mut self) -> Self {
        self.max_latency_us = u64::MAX;
        self
    }
}

pub struct WarpSearch {
    config: BeamConfig,
    prices: ResourcePrices,
}

impl WarpSearch {
    pub const fn new(config: BeamConfig) -> Self {
        Self {
            config,
            prices: ResourcePrices {
                network_micros: 0,
                storage_micros: 0,
                cpu_micros: 0,
                request_micros: 0,
            },
        }
    }

    pub const fn with_prices(mut self, prices: ResourcePrices) -> Self {
        self.prices = prices;
        self
    }

    pub fn choose_first(&self, nodes: &[ActionNode], budget: HardBudget) -> SearchDecision {
        let mut scorer = |actions: &[ActionNode]| static_score(actions, self.prices);
        self.choose(nodes, budget, &mut scorer)
    }

    pub fn choose_first_simulated(
        &self,
        nodes: &[ActionNode],
        budget: HardBudget,
        context: &mut TwinSearchContext<'_>,
    ) -> SearchDecision {
        let mut scorer = |actions: &[ActionNode]| context.score(actions);
        self.choose(nodes, budget, &mut scorer)
    }

    pub(crate) fn choose_first_recorded<F>(
        &self,
        nodes: &[ActionNode],
        budget: HardBudget,
        scorer: &mut F,
    ) -> SearchDecision
    where
        F: FnMut(&[ActionNode]) -> i64,
    {
        self.choose(nodes, budget, scorer)
    }

    pub(crate) fn choose_first_greedy(
        &self,
        nodes: &[ActionNode],
        budget: HardBudget,
        reason: SearchPruneReason,
    ) -> SearchDecision {
        self.greedy(nodes, budget, reason)
    }

    fn choose<F>(&self, nodes: &[ActionNode], budget: HardBudget, scorer: &mut F) -> SearchDecision
    where
        F: FnMut(&[ActionNode]) -> i64,
    {
        if self.config.max_latency_us == 0 {
            return self.greedy(nodes, budget, SearchPruneReason::PlannerLatency);
        }
        let fallback = budget.clone();
        self.beam(nodes, budget, scorer)
            .unwrap_or_else(|reason| self.greedy(nodes, fallback, reason))
    }

    fn beam<F>(
        &self,
        nodes: &[ActionNode],
        budget: HardBudget,
        scorer: &mut F,
    ) -> Result<SearchDecision, SearchPruneReason>
    where
        F: FnMut(&[ActionNode]) -> i64,
    {
        let started = Instant::now();
        let mut states = vec![State::new(budget)];
        let mut progress = SearchProgress::new(scorer);
        for _ in 0..self.config.depth {
            states = self.expand(nodes, states, &mut progress);
            if let Some(reason) = self.expired(started, progress.expansions) {
                return Err(reason);
            }
            if states.is_empty() {
                break;
            }
            progress.retained = states.clone();
        }
        let best = progress.best.take();
        let audit = progress.audit();
        Ok(decision(best, audit, false))
    }

    fn expand<F>(
        &self,
        nodes: &[ActionNode],
        states: Vec<State>,
        progress: &mut SearchProgress<'_, F>,
    ) -> Vec<State>
    where
        F: FnMut(&[ActionNode]) -> i64,
    {
        let mut next = Vec::new();
        for state in states {
            for node in nodes {
                match state.append(node, &mut *progress.scorer) {
                    Ok(Some(child)) => {
                        progress.retain_best(&child);
                        next.push(child);
                    }
                    Err(reason) => progress.prune(pruned_plan(&state, node.id, reason)),
                    Ok(None) => {}
                }
            }
        }
        next.sort_by(compare);
        next.drain(self.config.width.min(next.len())..)
            .for_each(|state| progress.prune_state(state, SearchPruneReason::BeamWidth));
        next
    }

    fn expired(&self, started: Instant, expansions: usize) -> Option<SearchPruneReason> {
        if expansions > self.config.max_expansions {
            Some(SearchPruneReason::ExpansionLimit)
        } else if started.elapsed().as_micros() > u128::from(self.config.max_latency_us) {
            Some(SearchPruneReason::PlannerLatency)
        } else {
            None
        }
    }
}

fn static_score(actions: &[ActionNode], prices: ResourcePrices) -> i64 {
    actions.iter().fold(0_i64, |sum, action| {
        sum.saturating_add(action.value.total(action.resources, prices))
    })
}
