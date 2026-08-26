use super::*;

pub(crate) fn deletion_plan(events: &[Event]) -> Option<QueryPlan> {
    dependent_deletion_plan(events).map(|dependent| dependent.plan)
}
