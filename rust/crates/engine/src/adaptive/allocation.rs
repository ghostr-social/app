use super::admission::admitted;
use super::allocation_evidence::{allocation, whole_allocation, AllocationInputs};
use super::allocation_geometry::{fit_to_budget, overlaps_planned, request_slices};
use super::plan::{AllocationPlan, AllocationReason};
use super::ranges::{missing, missing_playable};
use super::sources::best_origin;
use super::{CandidateSnapshot, MediaLayout, OriginHealth, PlayabilitySnapshot, PlayableRange};

/// Default upper bound for one origin range request. An interrupted
/// request forfeits its undelivered bytes, so oversized requests make
/// focus changes and failures expensive; per-sample micro-requests
/// instead pay a round trip per few kilobytes. Contiguous missing
/// extents are therefore repacked into slices no larger than the
/// snapshot's `request_slice_bytes`, for which this is the ceiling.
pub const REQUEST_SLICE_BYTES: u64 = 256 * 1024;

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
    let budget = append_timeline_probe(plan, snapshot, origin, &inputs);
    append_ranges(plan, snapshot, origin, AppendInputs { budget, ..inputs })
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

fn append_timeline_probe(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    origin: &OriginHealth,
    inputs: &AppendInputs<'_>,
) -> u64 {
    let Some(probe) = inputs.candidate.timeline_probe else {
        return inputs.budget;
    };
    let mut budget = inputs.budget;
    for playable in request_slices(
        missing_playable(inputs.candidate, probe),
        snapshot.request_slice_bytes,
    ) {
        if playable.bytes.len() > budget {
            break;
        }
        if overlaps_planned(plan, inputs.candidate, playable.bytes) {
            continue;
        }
        let work = allocation(
            snapshot,
            AllocationInputs {
                candidate: inputs.candidate,
                origin,
                playable,
                emergency: inputs.emergency,
                reason: Some(AllocationReason::MediaLayoutDiscovery),
                reservation_budget: budget,
            },
        );
        let reserved = work.request.reserved_network_bytes();
        let promoted = work.request.promotion().is_some();
        plan.allocations.push(work);
        budget = budget.saturating_sub(reserved);
        if promoted {
            break;
        }
    }
    budget
}

fn append_ranges(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    origin: &OriginHealth,
    inputs: AppendInputs<'_>,
) -> u64 {
    let mut budget = candidate_budget(snapshot, &inputs);
    let mut gained = 0;
    for available in request_slices(missing(inputs.candidate), snapshot.request_slice_bytes) {
        if budget == 0 {
            break;
        }
        if gained >= inputs.target_ms && !demanded(inputs.candidate, available.bytes) {
            continue;
        }
        let playable = fit_to_budget(available, budget);
        if overlaps_planned(plan, inputs.candidate, playable.bytes) {
            continue;
        }
        let work = allocation(
            snapshot,
            AllocationInputs {
                candidate: inputs.candidate,
                origin,
                playable,
                emergency: inputs.emergency,
                reason: inputs.reason,
                reservation_budget: budget,
            },
        );
        let reserved = work.request.reserved_network_bytes();
        let promoted = work.request.promotion().is_some();
        plan.allocations.push(work);
        gained = gained.saturating_add(playable.playable_ms);
        budget = budget.saturating_sub(reserved);
        if promoted {
            break;
        }
    }
    budget
}

/// Whether a slice covers bytes a live consumer is blocked on; those
/// are fetched regardless of how satisfied the playback reserve is.
fn demanded(candidate: &CandidateSnapshot, bytes: crate::ByteRange) -> bool {
    candidate
        .demanded
        .is_some_and(|wanted| wanted.start < bytes.end && bytes.start < wanted.end)
}

fn candidate_budget(snapshot: &PlayabilitySnapshot, inputs: &AppendInputs<'_>) -> u64 {
    if inputs.candidate.layout != MediaLayout::RequiresCompleteFile {
        return inputs.budget;
    }
    let missing_bytes: u64 = missing(inputs.candidate)
        .iter()
        .map(|playable| playable.bytes.len())
        .sum();
    inputs
        .budget
        .max(missing_bytes)
        .min(snapshot.storage.available_bytes())
}
