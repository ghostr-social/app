use crate::adaptive::{PlayabilitySnapshot, WarpPlanningDecision};
use crate::PostId;

pub(super) fn set_source(input: &mut PlayabilitySnapshot, index: usize, source: &str) {
    input.candidates[index].origins[0].source = source.to_owned();
}

pub(super) fn assert_origin_admission(
    decision: &WarpPlanningDecision,
    post: &PostId,
    expected: bool,
) {
    let ids: Vec<_> = decision
        .generated
        .actions
        .iter()
        .filter(|action| &action.node.post == post && action.node.resources.requests > 0)
        .map(|action| action.node.id)
        .collect();
    assert!(!ids.is_empty(), "fixture needs network work for {post:?}");
    assert_eq!(
        ids.iter()
            .any(|id| decision.admissible_action_ids.contains(id)),
        expected,
        "unexpected admission for {post:?}: {decision:#?}"
    );
}
