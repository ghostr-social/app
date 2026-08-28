use crate::adaptive::{
    AdaptivePlayabilityPolicy, InFlightAction, PlannerCommand, PlannerContext, RequestOccupancy,
    RetrievalRequest, WarpPlanner, WarpPlannerInput, WarpPlanningDecision,
};
use crate::origin_model::{
    AdmissionClaimTerminal, DecisionMode, MediaClass, OriginAdmissionIntent, OriginContext,
    OriginModel, OriginQuery, RequestMethod,
};
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

#[test]
fn one_cold_request_does_not_hide_demand_for_a_concurrency_trial() {
    let origins = origin_after_started_request();

    assert!(decision_with_origins(1, &origins).additional_request_slot_demanded);
    let followup = selected_request(&decision_with_origins(2, &origins)).requested_bytes();
    assert!(
        followup.start >= 64_000,
        "trial overlaps active work: {followup:?}"
    );
}

fn decision(capacity: usize) -> WarpPlanningDecision {
    decision_with_origins(capacity, &OriginModel::default())
}

fn decision_with_origins(capacity: usize, origins: &OriginModel) -> WarpPlanningDecision {
    let input = input(capacity);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_request_occupancy(RequestOccupancy::from_sources([SOURCE]))
        .with_soft_request_capacity(capacity as u16, capacity as u16, Vec::new());
    WarpPlanner::default().plan(WarpPlannerInput::new(&input, &base, origins, &context))
}

fn origin_after_started_request() -> OriginModel {
    let context = OriginContext::new(RequestMethod::PrefixGet, 64_000, MediaClass::ProgressiveMp4)
        .with_concurrency(1);
    let query = OriginQuery::new(SOURCE, context);
    let mut origins = OriginModel::default();
    let (_, claim) = origins
        .claim(
            &query,
            9_999,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        )
        .into_parts();
    origins.complete_claim(
        claim.expect("cold request owns exploration"),
        AdmissionClaimTerminal::StartedWithoutObservation,
    );
    origins
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
