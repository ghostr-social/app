use super::super::privacy::DecisionPrivacy;
use super::capture as action_capture;
use super::{
    RecordedPrunedSearchPlan, RecordedRetainedSearchPlan, RecordedSearchPruneReason,
    RecordedWarpAction, RecordedWarpSearch,
};
use crate::adaptive::{SearchPruneReason, WarpPlanningDecision};

const PLAN_LIMIT: usize = 64;
const ACTION_LIMIT: usize = 4;

pub(super) fn capture(
    value: &WarpPlanningDecision,
    privacy: &DecisionPrivacy,
) -> RecordedWarpSearch {
    let recorder = SearchRecorder { value, privacy };
    RecordedWarpSearch {
        committed_actions: value.search.committed_actions,
        used_greedy_fallback: value.search.used_greedy_fallback,
        chosen_plan: value
            .search
            .chosen_plan
            .as_ref()
            .map(|plan| recorder.retained(plan)),
        retained_plans: recorder.retained_plans(),
        retained_plans_total: value.search.retained_plans.len() as u64,
        recorded_pruned_plans: recorder.pruned_plans(),
        pruned_plan_events_total: value.search.pruned_plan_events_total,
        pruned_plan_sample_truncated: value.search.pruned_plan_sample_truncated,
        recorder_truncated_pruned_plans: value.search.pruned_plans.len() > PLAN_LIMIT,
        common_random_seed: value.common_random_seed,
    }
}

struct SearchRecorder<'a> {
    value: &'a WarpPlanningDecision,
    privacy: &'a DecisionPrivacy,
}

impl SearchRecorder<'_> {
    fn retained_plans(&self) -> Vec<RecordedRetainedSearchPlan> {
        self.value
            .search
            .retained_plans
            .iter()
            .take(PLAN_LIMIT)
            .map(|plan| self.retained(plan))
            .collect()
    }

    fn pruned_plans(&self) -> Vec<RecordedPrunedSearchPlan> {
        self.value
            .search
            .pruned_plans
            .iter()
            .take(PLAN_LIMIT)
            .map(|plan| self.pruned(plan))
            .collect()
    }

    fn retained(&self, plan: &crate::adaptive::RetainedSearchPlan) -> RecordedRetainedSearchPlan {
        RecordedRetainedSearchPlan {
            actions: self.actions(&plan.action_ids),
            actions_total: plan.action_ids.len() as u64,
            score_micros: plan.score_micros,
        }
    }

    fn pruned(&self, plan: &crate::adaptive::PrunedSearchPlan) -> RecordedPrunedSearchPlan {
        RecordedPrunedSearchPlan {
            actions: self.actions(&plan.action_ids),
            actions_total: plan.action_ids.len() as u64,
            reason: prune_reason(plan.reason),
        }
    }

    fn actions(&self, ids: &[u16]) -> Vec<RecordedWarpAction> {
        ids.iter()
            .take(ACTION_LIMIT)
            .map(|id| self.action(*id))
            .collect()
    }

    fn action(&self, id: u16) -> RecordedWarpAction {
        let item = self
            .value
            .generated
            .actions
            .iter()
            .find(|item| item.node.id == id)
            .expect("search action must be generated");
        action_capture::action(item, self.value.prices, self.privacy)
    }
}

fn prune_reason(value: SearchPruneReason) -> RecordedSearchPruneReason {
    match value {
        SearchPruneReason::HardBudget
        | SearchPruneReason::MutuallyExclusive
        | SearchPruneReason::ReserveUnderflow => feasibility_reason(value),
        SearchPruneReason::BeamWidth
        | SearchPruneReason::ExpansionLimit
        | SearchPruneReason::PlannerLatency => limit_reason(value),
    }
}

fn feasibility_reason(value: SearchPruneReason) -> RecordedSearchPruneReason {
    match value {
        SearchPruneReason::HardBudget => RecordedSearchPruneReason::HardBudget,
        SearchPruneReason::MutuallyExclusive => RecordedSearchPruneReason::MutuallyExclusive,
        SearchPruneReason::ReserveUnderflow => RecordedSearchPruneReason::ReserveUnderflow,
        _ => unreachable!("search prune reason partition"),
    }
}

fn limit_reason(value: SearchPruneReason) -> RecordedSearchPruneReason {
    match value {
        SearchPruneReason::BeamWidth => RecordedSearchPruneReason::BeamWidth,
        SearchPruneReason::ExpansionLimit => RecordedSearchPruneReason::ExpansionLimit,
        SearchPruneReason::PlannerLatency => RecordedSearchPruneReason::PlannerLatency,
        _ => unreachable!("search prune reason partition"),
    }
}
