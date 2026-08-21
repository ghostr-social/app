use super::action;
use crate::adaptive::decision::record::DecisionRecord;
use crate::adaptive::{
    DecisionAction, DecisionReplayStatus, RecordedPlannerRetryAvailability,
    RecordedRetainedSearchPlan, RecordedWarpAction, RecordedWarpDecision,
};

pub(super) fn verify(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    require(record.retained_plans.is_empty() && record.pruned.is_empty())?;
    verify_candidates(record, decision)?;
    verify_seed(record, decision)?;
    verify_retry_evidence(decision)?;
    verify_actions(decision)?;
    verify_search(decision)?;
    verify_selection(record, decision)
}

fn verify_retry_evidence(decision: &RecordedWarpDecision) -> Result<(), DecisionReplayStatus> {
    let evidence = &decision.retry_availability;
    require(evidence.iter().all(|item| {
        matches!(
            item.availability,
            RecordedPlannerRetryAvailability::Cooling { .. }
        )
    }))?;
    require(evidence.iter().enumerate().all(|(index, item)| {
        evidence[..index]
            .iter()
            .all(|prior| prior.post_id != item.post_id)
    }))
}

fn verify_candidates(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    let mut posts = Vec::new();
    for action in &decision.admissible_actions {
        if !posts.contains(&action.post_id) {
            posts.push(action.post_id.clone());
        }
    }
    require(posts == record.admissible_candidates)
}

fn verify_seed(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    require(record.random_seed == decision.search.common_random_seed)?;
    require(
        decision
            .evaluation
            .is_none_or(|evaluation| evaluation.common_random_seed == record.random_seed),
    )
}

fn verify_actions(decision: &RecordedWarpDecision) -> Result<(), DecisionReplayStatus> {
    let admitted = &decision.admissible_actions;
    let pruned = &decision.unattributed_pre_search_pruned_actions;
    require(admitted.iter().chain(pruned).all(action::coherent))?;
    require(unique(admitted) && unique(pruned))?;
    require(disjoint(admitted, pruned))?;
    require(dependencies_exist(admitted, pruned))
}

fn verify_search(decision: &RecordedWarpDecision) -> Result<(), DecisionReplayStatus> {
    let search = &decision.search;
    require(search.pruned_plan_events_total >= search.recorded_pruned_plans.len() as u64)?;
    require(
        search
            .chosen_plan
            .as_ref()
            .is_none_or(|plan| plan_matches(plan, decision)),
    )?;
    require(
        search
            .retained_plans
            .iter()
            .all(|plan| plan_matches(plan, decision)),
    )?;
    require(
        search
            .recorded_pruned_plans
            .iter()
            .all(|plan| actions_match(&plan.actions, decision)),
    )
}

fn verify_selection(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    match decision.selected.as_ref() {
        Some(selected) => verify_selected(record, decision, selected),
        None => verify_noop(record, decision),
    }
}

fn verify_selected(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
    selected: &RecordedWarpAction,
) -> Result<(), DecisionReplayStatus> {
    require(action::coherent(selected))?;
    require(decision.admissible_actions.contains(selected))?;
    require(record.chosen_action.as_ref() == Some(&project(selected)))?;
    require(decision.search.committed_actions == 1 && decision.evaluation.is_some())?;
    require(
        decision
            .search
            .chosen_plan
            .as_ref()
            .is_some_and(|plan| plan.actions.first() == Some(selected)),
    )
}

fn verify_noop(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    require(record.chosen_action.is_none())?;
    require(decision.search.committed_actions == 0)?;
    require(decision.search.chosen_plan.is_none())?;
    require(decision.evaluation.is_none())
}

fn project(action: &RecordedWarpAction) -> DecisionAction {
    let (request, source, bytes_start, bytes_end) = action.command.projection();
    DecisionAction {
        post_id: action.post_id.clone(),
        source_id: source.into(),
        request: request.into(),
        bytes_start,
        bytes_end,
        expected_playable_gain_ms: action.ready_playback_ms,
        utility_micros: action.static_score_micros,
        reason: "WarpSelected".into(),
        retained: false,
    }
}

fn plan_matches(plan: &RecordedRetainedSearchPlan, decision: &RecordedWarpDecision) -> bool {
    actions_match(&plan.actions, decision)
}

fn actions_match(actions: &[RecordedWarpAction], decision: &RecordedWarpDecision) -> bool {
    unique(actions)
        && actions
            .iter()
            .all(|action| decision.admissible_actions.contains(action))
        && dependencies_precede(actions)
}

fn dependencies_precede(actions: &[RecordedWarpAction]) -> bool {
    actions.iter().enumerate().all(|(index, action)| {
        action.dependencies.iter().all(|required| {
            actions[..index]
                .iter()
                .any(|prior| prior.planner_action_id == *required)
        })
    })
}

fn dependencies_exist(admitted: &[RecordedWarpAction], pruned: &[RecordedWarpAction]) -> bool {
    admitted.iter().chain(pruned).all(|action| {
        action.dependencies.iter().all(|required| {
            *required != action.planner_action_id
                && admitted
                    .iter()
                    .chain(pruned)
                    .any(|candidate| candidate.planner_action_id == *required)
        })
    })
}

fn unique(actions: &[RecordedWarpAction]) -> bool {
    actions.iter().enumerate().all(|(index, action)| {
        !actions[..index]
            .iter()
            .any(|prior| prior.planner_action_id == action.planner_action_id)
    })
}

fn disjoint(left: &[RecordedWarpAction], right: &[RecordedWarpAction]) -> bool {
    left.iter().all(|action| {
        right
            .iter()
            .all(|other| action.planner_action_id != other.planner_action_id)
    })
}

fn require(coherent: bool) -> Result<(), DecisionReplayStatus> {
    coherent
        .then_some(())
        .ok_or(DecisionReplayStatus::PlanMismatch)
}
