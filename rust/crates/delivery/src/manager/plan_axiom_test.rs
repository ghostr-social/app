use super::*;

/// Runs the pure engine planner over the manager's current picture.
/// Posts with no source left to try drop out entirely: they are
/// terminal, not work to reschedule on the next pass.
pub(crate) fn planned_work(state: &DeliveryState, inputs: &PlanInputs<'_>) -> PlannedWork {
    let mut planner = ghostr_engine::adaptive::WarpPlanner::default();
    let watch = ghostr_engine::watch_model::WatchModel::default();
    planned_work_with_planner(state, inputs, &mut planner, &watch)
}

pub(crate) fn planned_work_with_watch(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    watch: &ghostr_engine::watch_model::WatchModel,
) -> PlannedWork {
    let mut planner = ghostr_engine::adaptive::WarpPlanner::default();
    planned_work_with_planner(state, inputs, &mut planner, watch)
}
