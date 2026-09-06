use super::WarpPlannerInput;
use crate::adaptive::{ActionNode, Allocation, PreemptionAuthority};
use crate::RequestAuthority;

mod current_capacity;

pub(super) fn available(input: &WarpPlannerInput<'_>, node: &ActionNode) -> bool {
    if node.resources.requests == 0 {
        return true;
    }
    let Some(source) = missing_current_source(input) else {
        return true;
    };
    let occupancy = input.context.request_occupancy();
    let required = usize::from(node.resources.requests).saturating_add(1);
    if occupancy.total().saturating_add(required) > usize::from(input.context.limits.request_tokens)
    {
        return false;
    }
    let same_origin = RequestAuthority::from_url(source).as_ref() == node.request_authority();
    !same_origin
        || occupancy.authority_count(source).saturating_add(required)
            <= usize::from(input.context.limits.per_origin_requests)
}

fn missing_current_source<'a>(input: &WarpPlannerInput<'a>) -> Option<&'a str> {
    if !input
        .context
        .permits_request(&input.snapshot.playback.current)
    {
        return None;
    }
    input
        .base
        .allocations
        .iter()
        .filter(|work| work.post == input.snapshot.playback.current)
        .filter(|work| work.authority == PreemptionAuthority::PlaybackCritical)
        .find(|work| !covered_by_current(input, work))
        .map(|work| work.source.as_str())
        .or_else(|| missing_hls_source(input))
        .filter(|source| !current_capacity::owns_origin(input, source))
}

fn missing_hls_source<'a>(input: &WarpPlannerInput<'a>) -> Option<&'a str> {
    input
        .snapshot
        .hls_candidates
        .iter()
        .find(|candidate| candidate.post == input.snapshot.playback.current)
        .and_then(|candidate| {
            candidate.pending_request_source(
                input.snapshot.request_slice_bytes,
                input.context.segmented_storage_budget().available_bytes(),
            )
        })
}

fn covered_by_current(input: &WarpPlannerInput<'_>, work: &Allocation) -> bool {
    let planned = work.request.requested_bytes();
    input
        .snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.post == work.post)
        .flat_map(|candidate| &candidate.in_flight)
        .any(|active| {
            active.identity_current
                && !active.cancelling
                && active.source == work.source
                && active.effective_bytes.start < planned.end
                && planned.start < active.effective_bytes.end
        })
}
