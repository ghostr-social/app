use super::*;

impl WarpSearch {
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
}
