use super::{WarpPlanner, WarpPlannerInput};
use crate::adaptive::ResourceFeedback;

pub(super) fn observe(planner: &mut WarpPlanner, input: &WarpPlannerInput<'_>) {
    let before = planner.prices.prices();
    if let Some(current) = input
        .context
        .feedback
        .filter(|current| new_interval(planner.last_feedback, *current))
    {
        match current.price_snapshot {
            Some(snapshot) => {
                planner.prices =
                    crate::adaptive::ShadowPriceController::from_prices(snapshot.prices);
            }
            None => planner.prices.observe(current.actual, current.target),
        }
        planner.last_feedback = Some(current);
    }
    if planner.prices.prices() != before {
        planner.price_epoch = planner.price_epoch.saturating_add(1);
    }
    planner.twin.set_prices(planner.prices.prices());
}

fn new_interval(previous: Option<ResourceFeedback>, current: ResourceFeedback) -> bool {
    match (
        previous.and_then(|value| value.price_snapshot),
        current.price_snapshot,
    ) {
        (Some(previous), Some(current)) => current.cursor > previous.cursor,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (None, None) => previous.is_none_or(|value| current.revision > value.revision),
    }
}
