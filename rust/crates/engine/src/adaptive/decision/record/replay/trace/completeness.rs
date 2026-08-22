use crate::adaptive::{DecisionReplayStatus, RecordedRetainedSearchPlan, RecordedWarpDecision};

pub(super) fn verify(decision: &RecordedWarpDecision) -> Result<(), DecisionReplayStatus> {
    exact(
        decision.admissible_actions.len(),
        decision.admissible_actions_total,
    )?;
    exact(
        decision.unattributed_pre_search_pruned_actions.len(),
        decision.unattributed_pre_search_pruned_actions_total,
    )?;
    verify_search(decision)
}

fn verify_search(decision: &RecordedWarpDecision) -> Result<(), DecisionReplayStatus> {
    let search = &decision.search;
    exact(search.retained_plans.len(), search.retained_plans_total)?;
    require(!search.pruned_plan_sample_truncated)?;
    require(!search.recorder_truncated_pruned_plans)?;
    verify_plan(search.chosen_plan.as_ref())?;
    verify_plans(&search.retained_plans)?;
    verify_pruned(decision)
}

fn verify_plans(plans: &[RecordedRetainedSearchPlan]) -> Result<(), DecisionReplayStatus> {
    plans.iter().try_for_each(|plan| verify_plan(Some(plan)))
}

fn verify_pruned(decision: &RecordedWarpDecision) -> Result<(), DecisionReplayStatus> {
    decision
        .search
        .recorded_pruned_plans
        .iter()
        .try_for_each(|plan| exact(plan.actions.len(), plan.actions_total))
}

fn verify_plan(plan: Option<&RecordedRetainedSearchPlan>) -> Result<(), DecisionReplayStatus> {
    match plan {
        Some(plan) => exact(plan.actions.len(), plan.actions_total),
        None => Ok(()),
    }
}

fn exact(actual: usize, recorded: u64) -> Result<(), DecisionReplayStatus> {
    require(recorded == actual as u64)
}

fn require(complete: bool) -> Result<(), DecisionReplayStatus> {
    complete
        .then_some(())
        .ok_or(DecisionReplayStatus::AdvancedReplayUnavailable)
}
