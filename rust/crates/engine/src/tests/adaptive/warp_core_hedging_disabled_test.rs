use crate::adaptive::{
    ActionKind, ActivePlannerContext, AllocationPlan, HedgeInput, IdentityProof, InFlightAction,
    PlannerCommand, PlannerContext, RetrievalRequest, WarpPlanner, WarpPlannerInput,
    WholeBodyContract, WholeFetchReason,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

#[test]
fn core_does_not_race_whole_responses_without_rollout_evidence() {
    let mut state = snapshot(1, 8_000_000, 1_000, 20);
    let source = state.candidates[0].origins[0].source.clone();
    let id = ActionId::new(17);
    let mut active = InFlightAction::range(id, ByteRange::new(0, 64_000), source, 20_000, true);
    active.request = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact {
            expected_bytes: 64_000,
        },
        reason: WholeFetchReason::PlannedCompletion,
    };
    state.candidates[0].in_flight.push(active);
    let hedge = HedgeInput::new(
        id,
        ActionKind::FetchWhole {
            maximum_bytes: 64_000,
        },
    )
    .with_timing(1_000, 900)
    .with_value(5_000, 1_000);
    let active = ActivePlannerContext::new(id, state.candidates[0].post.clone()).with_hedge(
        hedge,
        IdentityProof::IndependentWhole,
        "https://mirror.example/media",
    );
    let context = PlannerContext::explicitly_unavailable(&state).with_active(active);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &AllocationPlan::default(),
        &OriginModel::default(),
        &context,
    ));
    assert!(!decision
        .generated
        .actions
        .iter()
        .any(|action| { matches!(action.command, PlannerCommand::Hedge { .. }) }));
}
