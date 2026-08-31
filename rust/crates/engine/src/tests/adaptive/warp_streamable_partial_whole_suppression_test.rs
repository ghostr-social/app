use super::streamable_partial_fixture::{partial_state, TOTAL};
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, MediaLayout, PlannerCapability, PlannerCommand,
    PlannerContext, RetrievalRequest, TransformCapability, TransformKind, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::{ByteRange, PostId};

#[test]
fn promotable_partial_range_does_not_offer_duplicate_whole_fetch() {
    let mut state = partial_state(TOTAL);
    state.request_slice_bytes = 65_536;
    state.playback.current = PostId::new("p1");
    state.playback.buffer_ahead_ms = 0;
    state.candidates[1].layout = MediaLayout::Unknown;
    let decision = plan(&state, PlannerCapability::reported(true, None, 1));

    assert_eq!(
        decision.generated.actions.len(),
        1,
        "{:#?}",
        decision.generated.actions
    );
    let PlannerCommand::Transfer(allocation) = &decision.generated.actions[0].command else {
        panic!("range transfer");
    };
    assert!(matches!(allocation.request,
        RetrievalRequest::FetchRange { bytes, promotion: Some(grant) }
            if bytes == ByteRange::new(65_536, 131_072) && grant.maximum_bytes == TOTAL));
}

#[test]
fn multi_slice_or_transform_completion_preserves_whole_crossover() {
    let large = partial_state(3_750_000);
    assert!(has_whole(&plan(
        &large,
        PlannerCapability::reported(true, None, 1),
    )));
    let mut blocked = partial_state(TOTAL);
    blocked.candidates[1].direct_playback_blocked = true;
    let transform = TransformCapability::new(TransformKind::Remux, 17, TOTAL);
    let decision = plan(
        &blocked,
        PlannerCapability::reported(false, Some(transform), 1),
    );
    assert!(has_whole(&decision));
    assert!(decision
        .generated
        .actions
        .iter()
        .any(|action| matches!(action.node.kind, ActionKind::Transform(_))));
}

fn plan(
    state: &crate::adaptive::PlayabilitySnapshot,
    capability: PlannerCapability,
) -> crate::adaptive::WarpPlanningDecision {
    let base = AdaptivePlayabilityPolicy.plan(state);
    let context = PlannerContext::explicitly_unavailable(state)
        .with_capability(&state.candidates[1].post, capability);
    WarpPlanner::default().plan(WarpPlannerInput::new(
        state,
        &base,
        &OriginModel::default(),
        &context,
    ))
}

fn has_whole(decision: &crate::adaptive::WarpPlanningDecision) -> bool {
    decision.generated.actions.iter().any(|action| {
        action.node.post == PostId::new("p1")
            && matches!(action.node.kind, ActionKind::FetchWhole { .. })
    })
}
