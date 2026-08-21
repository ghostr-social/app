use crate::adaptive::{
    AllocationPlan, BeamConfig, ControlMode, DecisionPrivacy, DecisionRecord, PlannerCapability,
    PlannerContext, RequestOccupancy, ShadowPrices, TransformCapability, TransformKind,
    WarpDecisionRecordInput, WarpPlanner, WarpPlannerConfig, WarpPlannerInput,
    WarpPlanningDecision,
};
use crate::origin_model::{
    ColdStartPrior, ColdStartSelector, MediaClass, OriginContext, OriginModel, OriginObservation,
    OriginQuery, RequestMethod,
};
use crate::tests::adaptive_support::snapshot;

pub(super) fn planned() -> (crate::adaptive::PlayabilitySnapshot, WarpPlanningDecision) {
    let state = rescue_state();
    let base = safety_plan();
    let context = rescue_context(&state);
    let origins = reliable_origin();
    let decision = WarpPlanner::new(replay_config())
        .plan(WarpPlannerInput::new(&state, &base, &origins, &context));
    (state, decision)
}

fn rescue_state() -> crate::adaptive::PlayabilitySnapshot {
    let mut state = snapshot(1, 20_000_000, 1_000, 20);
    state.commitment_ms = 1_000_000_000;
    state
}

fn safety_plan() -> AllocationPlan {
    AllocationPlan {
        mode: ControlMode::Safety,
        ..AllocationPlan::default()
    }
}

fn rescue_context(state: &crate::adaptive::PlayabilitySnapshot) -> PlannerContext {
    let mut context = PlannerContext::explicitly_unavailable(state)
        .with_capability(
            state.candidates[0].post.clone(),
            PlannerCapability::reported(
                false,
                Some(TransformCapability::new(TransformKind::Remux, 17, 128_000)),
                1,
            ),
        )
        .with_request_occupancy(RequestOccupancy::from_sources([
            "https://active.example/already-active",
        ]));
    context.limits.cpu_ms = 17;
    context.limits.request_tokens = 3;
    context.limits.per_origin_requests = 2;
    context
}

fn replay_config() -> WarpPlannerConfig {
    WarpPlannerConfig {
        beam: BeamConfig::new(2, 8, 64, u64::MAX),
        ..WarpPlannerConfig::default()
    }
}

fn reliable_origin() -> OriginModel {
    let mut model = OriginModel::default();
    model.register_cold_start(
        ColdStartSelector::default().with_method(RequestMethod::FullGet),
        ColdStartPrior::new(1_000_000_000.0, 0.1, 1, 100_000_000),
    );
    let context = OriginContext::new(
        RequestMethod::FullGet,
        3_750_000,
        MediaClass::ProgressiveMp4,
    )
    .with_observed_at_ms(10_000);
    let query = OriginQuery::new("https://origin.example/media", context);
    for _ in 0..10_000 {
        model.observe(OriginObservation::success(query.clone(), 10_000));
    }
    model
}

pub(super) fn record(
    state: &crate::adaptive::PlayabilitySnapshot,
    decision: &WarpPlanningDecision,
) -> DecisionRecord {
    DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 91,
        snapshot: state,
        decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([29; 32]),
    })
}
