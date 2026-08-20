mod output;
mod state;

pub use output::{PrunedSearchPlan, RetainedSearchPlan, SearchDecision, SearchPruneReason};

use self::output::{decision, pruned_plan};
use self::state::{compare, State};
use super::{ActionNode, DigitalTwin, HardBudget, ResourcePrices, TwinEpochs, TwinState};
use std::time::Instant;

const MAX_AUDIT_PLANS: usize = 64;

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
}

pub struct TwinSearchContext<'a> {
    twin: &'a mut DigitalTwin,
    state: TwinState,
    epochs: TwinEpochs,
}

impl<'a> TwinSearchContext<'a> {
    pub fn new(twin: &'a mut DigitalTwin, state: TwinState, epochs: TwinEpochs) -> Self {
        Self {
            twin,
            state,
            epochs,
        }
    }

    fn score(&mut self, actions: &[ActionNode]) -> i64 {
        self.twin
            .evaluate(&self.state, actions, self.epochs)
            .expected_score_micros
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
        Ok(decision(
            progress.best,
            progress.retained,
            progress.pruned,
            false,
        ))
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

    fn greedy(
        &self,
        nodes: &[ActionNode],
        budget: HardBudget,
        reason: SearchPruneReason,
    ) -> SearchDecision {
        let mut scorer = |actions: &[ActionNode]| static_score(actions, self.prices);
        let best = nodes
            .iter()
            .filter_map(|node| {
                State::new(budget.clone())
                    .append(node, &mut scorer)
                    .ok()
                    .flatten()
            })
            .filter(|state| state.score > 0)
            .min_by(compare);
        let retained = best.clone().into_iter().collect();
        let pruned = vec![PrunedSearchPlan {
            action_ids: Vec::new(),
            reason,
        }];
        decision(best, retained, pruned, true)
    }
}

struct SearchProgress<'a, F> {
    scorer: &'a mut F,
    expansions: usize,
    best: Option<State>,
    retained: Vec<State>,
    pruned: Vec<PrunedSearchPlan>,
}

impl<'a, F> SearchProgress<'a, F> {
    fn new(scorer: &'a mut F) -> Self {
        Self {
            scorer,
            expansions: 0,
            best: None,
            retained: Vec::new(),
            pruned: Vec::new(),
        }
    }

    fn retain_best(&mut self, child: &State) {
        self.expansions += 1;
        if child.score > 0
            && self
                .best
                .as_ref()
                .is_none_or(|old| compare(child, old).is_lt())
        {
            self.best = Some(child.clone());
        }
    }

    fn prune_state(&mut self, state: State, reason: SearchPruneReason) {
        self.prune(PrunedSearchPlan {
            action_ids: state.sequence.iter().map(|node| node.id).collect(),
            reason,
        });
    }

    fn prune(&mut self, plan: PrunedSearchPlan) {
        if self.pruned.len() < MAX_AUDIT_PLANS && !self.pruned.contains(&plan) {
            self.pruned.push(plan);
        }
    }
}

fn static_score(actions: &[ActionNode], prices: ResourcePrices) -> i64 {
    actions.iter().fold(0_i64, |sum, action| {
        sum.saturating_add(action.value.total(action.resources, prices))
    })
}
