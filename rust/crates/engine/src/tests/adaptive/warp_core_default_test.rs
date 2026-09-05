use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, ResourceFeedback, ResourceObservation,
    ResourcePrices, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn core_dispatches_without_running_unvalidated_optimization() {
    let state = snapshot(4, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state).with_feedback(ResourceFeedback {
        revision: 1,
        actual: ResourceObservation::new(200, 120, 20, 4),
        target: ResourceObservation::new(100, 100, 10, 2),
        price_snapshot: None,
    });
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert!(
        decision.selected.is_some(),
        "core must still protect playback"
    );
    assert!(
        decision.evaluation.is_none(),
        "CORE/3 must not run a digital twin"
    );
    assert_eq!(
        decision.prices,
        ResourcePrices::default(),
        "adaptive prices require rollout evidence"
    );
    assert!(
        decision.search.used_greedy_fallback,
        "CORE/3 must use deterministic dispatch"
    );
}

#[test]
fn core_never_generates_payload_outside_the_two_item_forward_window() {
    let state = snapshot(9, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    for action in decision.generated.actions {
        if let crate::adaptive::PlannerCommand::Transfer(transfer) = action.command {
            let candidate = state
                .candidates
                .iter()
                .find(|item| item.post == transfer.post)
                .expect("fixture");
            assert!(
                (0..=2).contains(&candidate.feed_offset.value()),
                "{:?}",
                transfer.post
            );
        }
    }
}
