use super::SearchPruneReason;
use crate::adaptive::{ActionKind, ActionNode, HardBudget};
use std::collections::BTreeSet;

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
        if self.sequence.iter().any(|chosen| conflicts(chosen, node)) {
            return Err(SearchPruneReason::MutuallyExclusive);
        }
        let mut child = self.clone();
        if !child
            .budget
            .consume(&node.resources, node.request_authority())
        {
            return Err(SearchPruneReason::HardBudget);
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

pub(super) fn compare(left: &State, right: &State) -> std::cmp::Ordering {
    right.score.cmp(&left.score).then_with(|| {
        left.sequence
            .iter()
            .map(|node| node.id)
            .cmp(right.sequence.iter().map(|node| node.id))
    })
}

fn conflicts(left: &ActionNode, right: &ActionNode) -> bool {
    left.post == right.post
        && (whole_fetch(&left.kind)
            || whole_fetch(&right.kind)
            || same_transfer_target(&left.kind, &right.kind))
}

fn whole_fetch(kind: &ActionKind) -> bool {
    matches!(kind, ActionKind::FetchWhole { .. })
}

fn same_transfer_target(left: &ActionKind, right: &ActionKind) -> bool {
    match (left, right) {
        (ActionKind::Prefix(left), ActionKind::Prefix(right))
        | (ActionKind::Tail(left), ActionKind::Tail(right))
        | (ActionKind::FetchRange(left), ActionKind::FetchRange(right))
        | (ActionKind::CacheUpgrade(left), ActionKind::CacheUpgrade(right)) => left == right,
        _ => false,
    }
}
