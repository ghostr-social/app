use super::State;
use crate::adaptive::ActionNode;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SearchPruneReason {
    HardBudget,
    MutuallyExclusive,
    BeamWidth,
    ExpansionLimit,
    PlannerLatency,
    ReserveUnderflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedSearchPlan {
    pub action_ids: Vec<u16>,
    pub score_micros: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrunedSearchPlan {
    pub action_ids: Vec<u16>,
    pub reason: SearchPruneReason,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchDecision {
    pub action: Option<ActionNode>,
    pub committed_actions: u8,
    pub used_greedy_fallback: bool,
    pub retained_plans: Vec<RetainedSearchPlan>,
    pub pruned_plans: Vec<PrunedSearchPlan>,
}

pub(super) fn decision(
    best: Option<State>,
    retained: Vec<State>,
    pruned: Vec<PrunedSearchPlan>,
    fallback: bool,
) -> SearchDecision {
    let action = best.and_then(|value| value.sequence.into_iter().next());
    SearchDecision {
        committed_actions: u8::from(action.is_some()),
        action,
        used_greedy_fallback: fallback,
        retained_plans: retained.into_iter().map(retained_plan).collect(),
        pruned_plans: pruned,
    }
}

pub(super) fn pruned_plan(state: &State, id: u16, reason: SearchPruneReason) -> PrunedSearchPlan {
    let mut action_ids: Vec<_> = state.sequence.iter().map(|node| node.id).collect();
    action_ids.push(id);
    PrunedSearchPlan { action_ids, reason }
}

fn retained_plan(state: State) -> RetainedSearchPlan {
    RetainedSearchPlan {
        action_ids: state.sequence.into_iter().map(|node| node.id).collect(),
        score_micros: state.score,
    }
}
