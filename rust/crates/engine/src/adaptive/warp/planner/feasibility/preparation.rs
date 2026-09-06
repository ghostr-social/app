use super::super::types::WarpPlannerInput;
use crate::adaptive::{ActionKind, ActionNode, NavigationPreparationPolicy};

mod request_room;

/// Eligible offscreen preparation does not authorize rescue reordering.
pub(super) fn may_prepare(
    input: &WarpPlannerInput<'_>,
    node: &ActionNode,
    policy: NavigationPreparationPolicy,
) -> bool {
    if policy.is_legacy() {
        return false;
    }
    (progressive(input, node) || hls(input, node) || knowledge(input, node, policy))
        && request_room::available(input, node)
}

fn progressive(input: &WarpPlannerInput<'_>, node: &ActionNode) -> bool {
    input.snapshot.candidates.iter().any(|candidate| {
        candidate.post == node.post
            && candidate.feed_offset.value() < 0
            && candidate.retrieval_eligible
    })
}

fn hls(input: &WarpPlannerInput<'_>, node: &ActionNode) -> bool {
    input.snapshot.hls_candidates.iter().any(|candidate| {
        candidate.post == node.post
            && candidate.feed_offset.value() < 0
            && candidate.pending().is_some()
    })
}

fn knowledge(
    input: &WarpPlannerInput<'_>,
    node: &ActionNode,
    policy: NavigationPreparationPolicy,
) -> bool {
    policy == NavigationPreparationPolicy::BidirectionalWindow
        && matches!(node.kind, ActionKind::Head)
        && input.snapshot.candidates.iter().any(|candidate| {
            candidate.post == node.post
                && candidate.retrieval_eligible
                && policy.includes_knowledge(candidate.feed_offset)
        })
}
