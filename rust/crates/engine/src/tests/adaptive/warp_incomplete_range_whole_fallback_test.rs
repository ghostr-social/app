use super::streamable_partial_fixture::{partial_state, TOTAL};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, MediaLayout, PlannerCommand, PlannerContext, PlayableRange,
    RetrievalRequest, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::ByteRange;

#[test]
fn a_short_discovery_range_does_not_dominate_complete_acquisition() {
    let mut state = partial_state(TOTAL);
    let candidate = &mut state.candidates[1];
    candidate.layout = MediaLayout::Unknown;
    candidate.startup = None;
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, 80_000),
        playable_ms: 1,
    }];
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &PlannerContext::explicitly_unavailable(&state),
    ));

    assert!(
        decision.generated.actions.iter().any(|action| {
            action.node.post == state.candidates[1].post
                && matches!(&action.command, PlannerCommand::Transfer(work)
                if matches!(work.request, RetrievalRequest::FetchWhole { .. }))
        }),
        "an incomplete range must not remove the ordinary whole-object route"
    );
}
