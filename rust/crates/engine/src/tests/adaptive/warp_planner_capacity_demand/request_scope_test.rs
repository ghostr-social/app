use crate::adaptive::{
    AdaptivePlayabilityPolicy, InFlightAction, PlannerCommand, PlannerContext, RequestOccupancy,
    RetrievalRequest, WarpPlanner, WarpPlannerInput, WarpPlanningDecision,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

const SOURCE: &str = "https://origin.example/media";

#[test]
fn scoped_capacity_growth_selects_only_the_disjoint_followup() {
    assert!(decision(1).additional_request_slot_demanded);
    let followup = selected_request(&decision(2)).requested_bytes();
    assert!(
        followup.start >= 64_000,
        "followup overlaps the owned prefix: {followup:?}"
    );
}

fn decision(capacity: usize) -> WarpPlanningDecision {
    let input = input(capacity);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_request_occupancy(RequestOccupancy::from_sources([SOURCE]))
        .with_soft_request_capacity(capacity as u16, capacity as u16, Vec::new());
    WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ))
}

fn input(capacity: usize) -> crate::adaptive::PlayabilitySnapshot {
    let mut input = snapshot(1, 20_000_000, 8_000, 0);
    input.network.connection_capacity = capacity;
    input.network.connection_ceiling = 2;
    input.network.per_authority_request_limit = 2;
    input.candidates[0].in_flight.push(InFlightAction::range(
        ActionId::new(1),
        ByteRange::new(0, 64_000),
        SOURCE,
        20_000,
        true,
    ));
    input
}

fn selected_request(decision: &WarpPlanningDecision) -> RetrievalRequest {
    match &decision.selected.as_ref().expect("selected action").command {
        PlannerCommand::Transfer(allocation) => allocation.request,
        command => panic!("expected transfer, got {command:?}"),
    }
}
