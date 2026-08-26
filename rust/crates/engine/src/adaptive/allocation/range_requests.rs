use super::AppendInputs;
use crate::adaptive::allocation_evidence::{allocation, AllocationInputs};
use crate::adaptive::allocation_geometry::{fit_to_budget, overlaps_planned, request_slices};
use crate::adaptive::plan::{AllocationPlan, AllocationReason};
use crate::adaptive::ranges::{missing, missing_playable};
use crate::adaptive::{CandidateSnapshot, MediaLayout, OriginHealth, PlayabilitySnapshot};

pub(super) fn append(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    origin: &OriginHealth,
    inputs: AppendInputs<'_>,
) -> u64 {
    let budget = append_timeline_probe(plan, snapshot, origin, &inputs);
    append_media_ranges(plan, snapshot, origin, AppendInputs { budget, ..inputs })
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
    let ranges = missing_playable(inputs.candidate, probe);
    append_probe_slices(plan, snapshot, origin, inputs, ranges)
}

fn append_probe_slices(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    origin: &OriginHealth,
    inputs: &AppendInputs<'_>,
    ranges: Vec<crate::adaptive::PlayableRange>,
) -> u64 {
    let mut budget = inputs.budget;
    for playable in request_slices(ranges, snapshot.request_slice_bytes) {
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

fn append_media_ranges(
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
    let missing_bytes = missing(inputs.candidate)
        .iter()
        .map(|playable| playable.bytes.len())
        .sum::<u64>();
    inputs
        .budget
        .max(missing_bytes)
        .min(snapshot.storage.available_bytes())
}
