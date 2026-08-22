use crate::adaptive::{
    AdaptivePlayabilityPolicy, InFlightAction, PlannerCapability, PlannerContext, ResourceFeedback,
    ResourceObservation, TransformCapability, TransformKind, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::tests::support::set_reliable_total_bytes;
use crate::{ActionId, ByteRange};

#[test]
fn degraded_dependency_root_request_demands_the_slot() {
    let decision = decision(true, false);
    assert!(decision.reserve.degraded);
    assert!(decision.selected.is_none());
    assert!(decision.additional_request_slot_demanded);
}

#[test]
fn degraded_positive_request_demand_is_reported() {
    let decision = decision(false, false);
    assert!(decision.reserve.degraded);
    assert!(decision.selected.is_none());
    assert!(decision.additional_request_slot_demanded);
}

#[test]
fn degraded_least_risk_request_demand_is_reported() {
    let decision = decision(false, true);
    assert!(decision.reserve.degraded);
    assert!(decision.selected.is_none());
    assert!(decision.additional_request_slot_demanded);
}

fn decision(transform: bool, priced_out: bool) -> crate::adaptive::WarpPlanningDecision {
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.commitment_ms = 0;
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 2;
    input.network.per_authority_request_limit = 2;
    set_reliable_total_bytes(&mut input.candidates[0], 128_000, input.observed_at_ms);
    input.candidates[0].in_flight.push(InFlightAction::range(
        ActionId::new(1),
        ByteRange::new(0, 64_000),
        "https://origin.example/media",
        20_000,
        true,
    ));
    let post = input.candidates[0].post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let mut context = PlannerContext::explicitly_unavailable(&input);
    if transform {
        context = context.with_capability(
            post,
            PlannerCapability::reported(
                false,
                Some(TransformCapability::new(TransformKind::Remux, 1, 1_000)),
                1,
            ),
        );
    }
    if priced_out {
        context = context.with_feedback(ResourceFeedback {
            revision: 1,
            actual: ResourceObservation::new(0, 0, 0, u64::MAX),
            target: ResourceObservation::new(1, 1, 1, 1),
            price_snapshot: None,
        });
    }
    context.limits.cpu_ms = 1;
    WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ))
}
