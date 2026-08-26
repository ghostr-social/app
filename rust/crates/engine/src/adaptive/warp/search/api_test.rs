use super::{static_score, WarpSearch};
use crate::adaptive::{ActionNode, HardBudget, SearchDecision};

impl WarpSearch {
    pub(crate) fn choose_first(&self, nodes: &[ActionNode], budget: HardBudget) -> SearchDecision {
        let mut scorer = |actions: &[ActionNode]| static_score(actions, self.prices);
        self.choose(nodes, budget, &mut scorer)
    }
}
