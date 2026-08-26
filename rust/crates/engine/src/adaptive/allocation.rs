use super::admission::admitted;
use super::allocation_evidence::{allocation, whole_allocation, AllocationInputs};
use super::allocation_geometry::{fit_to_budget, overlaps_planned};
use super::plan::{AllocationPlan, AllocationReason};
use super::ranges::missing;
use super::sources::best_origin;
use super::{CandidateSnapshot, MediaLayout, OriginHealth, PlayabilitySnapshot, PlayableRange};

mod range_requests;

/// Default upper bound for one origin range request.
///
/// An interrupted request forfeits its undelivered bytes, so oversized
/// requests make focus changes and failures expensive; per-sample micro-requests
/// instead pay a round trip per few kilobytes.
///
/// Contiguous missing
/// extents are therefore repacked into slices no larger than the
/// snapshot's `request_slice_bytes`, for which this is the ceiling.
pub const REQUEST_SLICE_BYTES: u64 = 256 * 1024;

#[derive(Clone, Copy)]
pub(super) struct AppendInputs<'a> {
    pub(super) candidate: &'a CandidateSnapshot,
    pub(super) target_ms: u64,
    pub(super) emergency: bool,
    pub(super) budget: u64,
    pub(super) reason: Option<AllocationReason>,
}

pub(super) fn append_candidate(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    inputs: AppendInputs<'_>,
) -> u64 {
    if has_exclusive_action(inputs.candidate) {
        return inputs.budget;
    }
    let Some(origin) = best_origin(inputs.candidate) else {
        return inputs.budget;
    };
    if !admitted_for_reason(snapshot, &inputs, origin) {
        return inputs.budget;
    }
    if inputs.candidate.layout == MediaLayout::RequiresCompleteFile {
        return append_whole(plan, snapshot, origin, inputs);
    }
    if inputs.reason == Some(AllocationReason::MediaBootstrap) {
        return append_bootstrap(plan, snapshot, origin, inputs);
    }
    range_requests::append(plan, snapshot, origin, inputs)
}

fn has_exclusive_action(candidate: &CandidateSnapshot) -> bool {
    candidate.in_flight.iter().any(|active| {
        active.identity_current
            && matches!(
                active.request,
                super::RetrievalRequest::FetchWhole { .. }
                    | super::RetrievalRequest::FetchRange {
                        promotion: Some(_),
                        ..
                    }
            )
    })
}

fn append_bootstrap(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    origin: &OriginHealth,
    inputs: AppendInputs<'_>,
) -> u64 {
    let Some(playable) = bootstrap_missing(snapshot, &inputs) else {
        return inputs.budget;
    };
    if playable.bytes.is_empty() || overlaps_planned(plan, inputs.candidate, playable.bytes) {
        return inputs.budget;
    }
    let work = allocation(
        snapshot,
        AllocationInputs {
            candidate: inputs.candidate,
            origin,
            playable,
            emergency: inputs.emergency,
            reason: inputs.reason,
            reservation_budget: inputs.budget,
        },
    );
    let reserved = work.request.reserved_network_bytes();
    plan.allocations.push(work);
    inputs.budget.saturating_sub(reserved)
}

fn append_whole(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    origin: &OriginHealth,
    inputs: AppendInputs<'_>,
) -> u64 {
    let Some(total) = inputs.candidate.total_bytes else {
        return inputs.budget;
    };
    let bytes = crate::ByteRange::new(0, total);
    if total > inputs.budget || missing(inputs.candidate).is_empty() {
        return inputs.budget;
    }
    if overlaps_planned(plan, inputs.candidate, bytes) {
        return inputs.budget;
    }
    plan.allocations.push(whole_allocation(
        snapshot,
        AllocationInputs {
            candidate: inputs.candidate,
            origin,
            playable: PlayableRange {
                bytes,
                playable_ms: inputs.candidate.duration_ms,
            },
            emergency: inputs.emergency,
            reason: inputs.reason,
            reservation_budget: inputs.budget,
        },
    ));
    inputs.budget.saturating_sub(total)
}

fn bootstrap_missing(
    _snapshot: &PlayabilitySnapshot,
    inputs: &AppendInputs<'_>,
) -> Option<PlayableRange> {
    missing(inputs.candidate)
        .into_iter()
        .next()
        .map(|missing| fit_to_budget(missing, inputs.budget))
}

fn admitted_for_reason(
    snapshot: &PlayabilitySnapshot,
    inputs: &AppendInputs<'_>,
    origin: &OriginHealth,
) -> bool {
    match inputs.reason {
        Some(AllocationReason::MediaBootstrap) => inputs.candidate.needs_bootstrap(),
        _ => admitted(snapshot, inputs.candidate, origin),
    }
}
