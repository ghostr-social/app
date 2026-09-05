use super::{feasibility::FeasibleActions, WarpPlannerInput};
use crate::adaptive::ActionKind;

/// Blocked decoder input and reacquisition of evicted current bytes precede
/// speculative readiness, including cold returns before decoder byte demand.
/// Keep hard resource admission and the actual request envelope intact.
pub(super) fn constrain(
    input: &WarpPlannerInput<'_>,
    feasible: &FeasibleActions,
) -> Option<FeasibleActions> {
    let current = &input.snapshot.playback.current;
    let candidate = input
        .snapshot
        .candidates
        .iter()
        .find(|candidate| &candidate.post == current)?;
    let wanted = current_demand(input.snapshot, candidate)?;
    let nodes: Vec<_> = feasible
        .nodes
        .iter()
        .filter(|node| &node.post == current && supplies(&node.kind, wanted.start))
        .cloned()
        .collect();
    if nodes.is_empty() {
        return None;
    }
    Some(FeasibleActions {
        nodes,
        ..feasible.clone()
    })
}

fn current_demand(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    candidate: &crate::adaptive::CandidateSnapshot,
) -> Option<crate::ByteRange> {
    candidate.demanded.or_else(|| {
        if snapshot.playback.authority != crate::adaptive::CurrentAuthority::Canonical
            || candidate.recently_evicted.is_empty()
            || !crate::adaptive::resources::endangered(snapshot, candidate)
        {
            return None;
        }
        crate::adaptive::ranges::missing(candidate)
            .first()
            .map(|range| range.bytes)
    })
}

fn supplies(kind: &ActionKind, offset: u64) -> bool {
    match kind {
        ActionKind::Prefix(bytes) | ActionKind::Tail(bytes) | ActionKind::FetchRange(bytes) => {
            bytes.start <= offset && offset < bytes.end
        }
        ActionKind::FetchWhole { maximum_bytes } => offset < *maximum_bytes,
        _ => false,
    }
}
