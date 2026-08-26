use super::output::{decision, PrunedSearchPlan, SearchAudit};
use super::state::{compare, State};
use super::{static_score, ActionNode, HardBudget, SearchDecision, SearchPruneReason, WarpSearch};

impl WarpSearch {
    pub(super) fn greedy(
        &self,
        nodes: &[ActionNode],
        budget: &HardBudget,
        reason: SearchPruneReason,
    ) -> SearchDecision {
        let mut scorer = |actions: &[ActionNode]| static_score(actions, self.prices);
        let best = nodes
            .iter()
            .filter_map(|node| append(node, budget, &mut scorer))
            .filter(|state| state.score > 0)
            .min_by(compare);
        let audit = SearchAudit {
            retained: best.clone().into_iter().collect(),
            pruned: vec![PrunedSearchPlan {
                action_ids: Vec::new(),
                reason,
            }],
            prune_events_total: 1,
            pruned_sample_truncated: false,
        };
        decision(best, audit, true)
    }
}

fn append<F>(node: &ActionNode, budget: &HardBudget, scorer: &mut F) -> Option<State>
where
    F: FnMut(&[ActionNode]) -> i64,
{
    State::new(budget.clone())
        .append(node, scorer)
        .ok()
        .flatten()
}
