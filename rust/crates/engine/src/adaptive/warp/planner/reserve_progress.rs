use crate::adaptive::{
    ActionKind, ActionNode, AllocationPlan, AllocationReason, ControlMode, PlayabilitySnapshot,
    ReserveCandidateEvidence, ReserveCandidateState, RetrievalRequest,
};
use crate::PostId;

#[cfg(test)]
#[path = "reserve_progress/partial_overlap_test.rs"]
mod partial_overlap_test;

pub(super) fn action_ids(
    snapshot: &PlayabilitySnapshot,
    base: &AllocationPlan,
    nodes: &[ActionNode],
) -> Vec<u16> {
    let Some(post) = target_post(base) else {
        return Vec::new();
    };
    nodes
        .iter()
        .filter(|node| advances(snapshot, base, post, node))
        .map(|node| node.id)
        .collect()
}

pub(super) fn target_post(base: &AllocationPlan) -> Option<&PostId> {
    let target = first_deficit(base)?;
    if !matches!(
        target.state,
        ReserveCandidateState::Planned { .. } | ReserveCandidateState::Preparing { .. }
    ) {
        return None;
    }
    readiness_allocations(base, &target.post)
        .next()
        .map(|_| &target.post)
}

pub(super) fn underflow(base: &AllocationPlan) -> bool {
    base.mode == ControlMode::Safety && !base.ready_reserve.ordered_target_satisfied()
}

fn first_deficit(base: &AllocationPlan) -> Option<&ReserveCandidateEvidence> {
    underflow(base).then(|| {
        base.ready_reserve
            .candidates
            .iter()
            .take(base.ready_reserve.target)
            .find(|item| !matches!(item.state, ReserveCandidateState::Ready { .. }))
    })?
}

fn advances(
    snapshot: &PlayabilitySnapshot,
    base: &AllocationPlan,
    post: &PostId,
    node: &ActionNode,
) -> bool {
    if node.post != *post || !readiness_kind(&node.kind) {
        return false;
    }
    let Some(request) = node.request() else {
        return false;
    };
    readiness_allocations(base, post).any(|work| {
        request == work.request
            || (node.forecast.ready_playback_ms > 0 && completes_readiness(snapshot, post, request))
    })
}

fn completes_readiness(
    snapshot: &PlayabilitySnapshot,
    post: &PostId,
    request: RetrievalRequest,
) -> bool {
    let Some(candidate) = snapshot.candidates.iter().find(|item| &item.post == post) else {
        return false;
    };
    let mut before = candidate.present.clone();
    before.extend(candidate.in_flight.iter().filter_map(active_coverage));
    let mut after = before.clone();
    after.push(request.requested_bytes());
    crate::adaptive::reserve_model::readiness_ranges(candidate)
        .iter()
        .filter(|range| crate::adaptive::ranges::uncovered_bytes(**range, &before) > 0)
        .any(|range| crate::adaptive::ranges::uncovered_bytes(*range, &after) == 0)
}

fn active_coverage(active: &crate::adaptive::InFlightAction) -> Option<crate::ByteRange> {
    (active.identity_current && !active.cancelling).then_some(active.effective_bytes)
}

fn readiness_allocations<'a>(
    base: &'a AllocationPlan,
    post: &'a PostId,
) -> impl Iterator<Item = &'a crate::adaptive::Allocation> {
    base.allocations.iter().filter(move |work| {
        work.post == *post
            && matches!(
                work.reason,
                AllocationReason::MediaBootstrap | AllocationReason::NextStartability
            )
    })
}

fn readiness_kind(kind: &ActionKind) -> bool {
    matches!(
        kind,
        ActionKind::Prefix(_)
            | ActionKind::Tail(_)
            | ActionKind::FetchRange(_)
            | ActionKind::FetchWhole { .. }
    )
}
