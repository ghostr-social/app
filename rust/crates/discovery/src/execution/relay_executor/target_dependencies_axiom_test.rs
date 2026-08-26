use super::*;

pub(crate) fn target_plan_with_dependencies(
    events: &[Event],
) -> Option<(QueryPlan, Vec<BTreeSet<EventId>>, BTreeSet<EventId>)> {
    dependent_target_plan(events)
        .map(|dependent| (dependent.plan, dependent.dependencies, dependent.unplanned))
}
