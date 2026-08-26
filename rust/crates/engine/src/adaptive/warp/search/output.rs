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
    pub(crate) action_ids: Vec<u16>,
    pub(crate) reason: SearchPruneReason,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchDecision {
    pub action: Option<ActionNode>,
    pub chosen_plan: Option<RetainedSearchPlan>,
    pub committed_actions: u8,
    pub used_greedy_fallback: bool,
    pub retained_plans: Vec<RetainedSearchPlan>,
    pub pruned_plans: Vec<PrunedSearchPlan>,
    pub pruned_plan_events_total: u64,
    pub pruned_plan_sample_truncated: bool,
}

pub(super) struct SearchAudit {
    pub retained: Vec<State>,
    pub pruned: Vec<PrunedSearchPlan>,
    pub prune_events_total: u64,
    pub pruned_sample_truncated: bool,
}

pub(super) fn decision(best: Option<State>, audit: SearchAudit, fallback: bool) -> SearchDecision {
    let chosen_plan = best.clone().map(retained_plan);
    let action = best.and_then(|value| value.sequence.into_iter().next());
    SearchDecision {
        committed_actions: u8::from(action.is_some()),
        action,
        chosen_plan,
        used_greedy_fallback: fallback,
        retained_plans: audit.retained.into_iter().map(retained_plan).collect(),
        pruned_plans: audit.pruned,
        pruned_plan_events_total: audit.prune_events_total,
        pruned_plan_sample_truncated: audit.pruned_sample_truncated,
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
