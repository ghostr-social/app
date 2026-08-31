use super::streamable_partial_fixture::{partial_state, TOTAL};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, MediaLayout, PlannerCapability, PlannerCommand, PlannerContext,
    RetrievalRequest, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::{
    ErrorReason, MediaClass, OriginContext, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};
use crate::tests::adaptive_support::healthy_origin;
use crate::PostId;

const RANGE_SOURCE: &str = "https://range.example/video.mp4";
const WHOLE_SOURCE: &str = "https://whole.example/video.mp4";

#[test]
fn promoted_range_does_not_hide_a_different_source_whole_rescue() {
    let mut state = partial_state(TOTAL);
    state.observed_at_ms = 9_000;
    state.request_slice_bytes = 65_536;
    state.playback.current = PostId::new("p1");
    state.playback.buffer_ahead_ms = 0;
    let candidate = &mut state.candidates[1];
    candidate.layout = MediaLayout::Unknown;
    candidate.preferred_source = Some(RANGE_SOURCE.into());
    candidate.origins = vec![
        healthy_origin(RANGE_SOURCE, 40_000_000, 10),
        healthy_origin(WHOLE_SOURCE, 20_000_000, 20),
    ];
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state).with_capability(
        &state.candidates[1].post,
        PlannerCapability::reported(true, None, 1),
    );
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &full_get_open_on_range_source(),
        &context,
    ));

    assert!(has_promoted_range(&decision));
    assert!(has_whole(&decision));
}

fn has_promoted_range(decision: &crate::adaptive::WarpPlanningDecision) -> bool {
    decision.generated.actions.iter().any(|action| {
        let PlannerCommand::Transfer(allocation) = &action.command else {
            return false;
        };
        allocation.source == RANGE_SOURCE
            && matches!(allocation.request,
                RetrievalRequest::FetchRange { promotion: Some(grant), .. }
                    if grant.maximum_bytes >= TOTAL)
    })
}

fn has_whole(decision: &crate::adaptive::WarpPlanningDecision) -> bool {
    decision.generated.actions.iter().any(|action| {
        let PlannerCommand::Transfer(allocation) = &action.command else {
            return false;
        };
        allocation.source == WHOLE_SOURCE
            && matches!(allocation.request, RetrievalRequest::FetchWhole { .. })
    })
}

fn full_get_open_on_range_source() -> OriginModel {
    let mut model = OriginModel::default();
    for at_ms in 7_000..7_003 {
        let context = OriginContext::new(RequestMethod::FullGet, TOTAL, MediaClass::ProgressiveMp4);
        model.observe(&OriginObservation::failure(
            OriginQuery::new(RANGE_SOURCE, context),
            at_ms,
            ErrorReason::Timeout,
        ));
    }
    model
}
