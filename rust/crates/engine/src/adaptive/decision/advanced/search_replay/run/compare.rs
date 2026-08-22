use super::super::super::{
    RecordedPrunedSearchPlan, RecordedRetainedSearchPlan, RecordedSearchPruneReason,
    RecordedWarpAction, RecordedWarpSearch,
};
use crate::adaptive::{SearchDecision, SearchPruneReason};

pub(super) fn matches(actual: &SearchDecision, expected: &RecordedWarpSearch) -> bool {
    metadata_matches(actual, expected)
        && plan_matches(actual.chosen_plan.as_ref(), expected.chosen_plan.as_ref())
        && retained_match(actual, expected)
        && pruned_match(actual, expected)
}

fn metadata_matches(actual: &SearchDecision, expected: &RecordedWarpSearch) -> bool {
    actual.committed_actions == expected.committed_actions
        && actual.used_greedy_fallback == expected.used_greedy_fallback
        && actual.pruned_plan_events_total == expected.pruned_plan_events_total
        && actual.pruned_plan_sample_truncated == expected.pruned_plan_sample_truncated
}

fn retained_match(actual: &SearchDecision, expected: &RecordedWarpSearch) -> bool {
    actual.retained_plans.len() as u64 == expected.retained_plans_total
        && actual.retained_plans.len() == expected.retained_plans.len()
        && actual
            .retained_plans
            .iter()
            .zip(&expected.retained_plans)
            .all(|(actual, expected)| plan_matches(Some(actual), Some(expected)))
}

fn pruned_match(actual: &SearchDecision, expected: &RecordedWarpSearch) -> bool {
    actual.pruned_plans.len() == expected.recorded_pruned_plans.len()
        && actual
            .pruned_plans
            .iter()
            .zip(&expected.recorded_pruned_plans)
            .all(|(actual, expected)| pruned_plan_matches(actual, expected))
}

fn plan_matches(
    actual: Option<&crate::adaptive::RetainedSearchPlan>,
    expected: Option<&RecordedRetainedSearchPlan>,
) -> bool {
    match (actual, expected) {
        (Some(actual), Some(expected)) => {
            actual.score_micros == expected.score_micros
                && action_ids(&expected.actions) == actual.action_ids
        }
        (None, None) => true,
        _ => false,
    }
}

fn pruned_plan_matches(
    actual: &crate::adaptive::PrunedSearchPlan,
    expected: &RecordedPrunedSearchPlan,
) -> bool {
    action_ids(&expected.actions) == actual.action_ids && reason(actual.reason) == expected.reason
}

fn action_ids(actions: &[RecordedWarpAction]) -> Vec<u16> {
    actions
        .iter()
        .map(|action| action.planner_action_id)
        .collect()
}

fn reason(value: SearchPruneReason) -> RecordedSearchPruneReason {
    match value {
        SearchPruneReason::HardBudget
        | SearchPruneReason::MutuallyExclusive
        | SearchPruneReason::ReserveUnderflow => feasibility_reason(value),
        SearchPruneReason::BeamWidth => RecordedSearchPruneReason::BeamWidth,
        SearchPruneReason::ExpansionLimit => RecordedSearchPruneReason::ExpansionLimit,
        SearchPruneReason::PlannerLatency => RecordedSearchPruneReason::PlannerLatency,
    }
}

fn feasibility_reason(value: SearchPruneReason) -> RecordedSearchPruneReason {
    match value {
        SearchPruneReason::HardBudget => RecordedSearchPruneReason::HardBudget,
        SearchPruneReason::MutuallyExclusive => RecordedSearchPruneReason::MutuallyExclusive,
        SearchPruneReason::ReserveUnderflow => RecordedSearchPruneReason::ReserveUnderflow,
        _ => unreachable!("only feasibility reasons are routed here"),
    }
}
