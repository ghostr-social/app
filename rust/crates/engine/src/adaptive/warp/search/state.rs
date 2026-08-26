use super::super::budget::BudgetDenial;
use super::SearchPruneReason;
use crate::adaptive::{ActionNode, HardBudget};
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "state/dependency_conflict_test.rs"]
mod dependency_conflict_test;

#[cfg(test)]
#[path = "state/rescue_reserve_test.rs"]
mod rescue_reserve_test;

#[derive(Clone)]
pub(super) struct State {
    pub budget: HardBudget,
    pub selected: BTreeSet<u16>,
    pub sequence: Vec<ActionNode>,
    pub score: i64,
}

impl State {
    pub fn new(budget: HardBudget) -> Self {
        Self {
            budget,
            selected: BTreeSet::new(),
            sequence: Vec::new(),
            score: 0,
        }
    }

    pub fn append<F>(
        &self,
        node: &ActionNode,
        scorer: &mut F,
    ) -> Result<Option<Self>, SearchPruneReason>
    where
        F: FnMut(&[ActionNode]) -> i64,
    {
        if !self.dependencies_met(node) {
            return Ok(None);
        }
        if node.conflicts_with(&self.sequence) {
            return Err(SearchPruneReason::MutuallyExclusive);
        }
        let mut child = self.clone();
        match child.budget.consume_action(node) {
            Ok(()) => {}
            Err(BudgetDenial::HardLimit) => return Err(SearchPruneReason::HardBudget),
            Err(BudgetDenial::RescueReserve) => return Err(SearchPruneReason::ReserveUnderflow),
        }
        child.selected.insert(node.id);
        child.sequence.push(node.clone());
        child.score = scorer(&child.sequence);
        Ok(Some(child))
    }

    fn dependencies_met(&self, node: &ActionNode) -> bool {
        !self.selected.contains(&node.id)
            && node.requires.iter().all(|id| self.selected.contains(id))
    }
}

pub(super) fn compare(left: &State, right: &State) -> core::cmp::Ordering {
    right.score.cmp(&left.score).then_with(|| {
        left.sequence
            .iter()
            .map(|node| node.id)
            .cmp(right.sequence.iter().map(|node| node.id))
    })
}
