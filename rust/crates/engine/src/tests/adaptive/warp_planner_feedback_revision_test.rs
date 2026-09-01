use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, ResourceFeedback, ResourceObservation, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn one_resource_sample_updates_shadow_prices_exactly_once() {
    let input = snapshot(2, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let first = feedback(7);
    let mut planner = WarpPlanner::default();
    let context = PlannerContext::explicitly_unavailable(&input).with_feedback(first);

    let once = planner.plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let repeated = planner.plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let next = PlannerContext::explicitly_unavailable(&input).with_feedback(feedback(8));
    let twice = planner.plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &next,
    ));

    assert_eq!(repeated.prices, once.prices);
    assert!(twice.prices.network_micros > once.prices.network_micros);
}

fn feedback(revision: u64) -> ResourceFeedback {
    ResourceFeedback {
        revision,
        actual: ResourceObservation::new(200, 0, 0, 0),
        target: ResourceObservation::new(100, 1, 0, 1),
        price_snapshot: None,
    }
}
