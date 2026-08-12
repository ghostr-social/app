use super::admission::admitted;
use super::allocation_evidence::{allocation, AllocationInputs};
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
}

pub(super) fn append_candidate(
    plan: &mut AllocationPlan,
    snapshot: &PlayabilitySnapshot,
    inputs: AppendInputs<'_>,
) -> u64 {
    let Some(origin) = best_origin(inputs.candidate) else {
        return inputs.budget;
    };
    if !admitted(snapshot, inputs.candidate, origin) {
        return inputs.budget;
    }
    let budget = append_timeline_probe(plan, snapshot, origin, &inputs);
    append_ranges(plan, snapshot, origin, AppendInputs { budget, ..inputs })
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
        plan.allocations.push(allocation(
            snapshot,
            AllocationInputs {
                candidate: inputs.candidate,
                origin,
                playable,
                emergency: inputs.emergency,
                reason: Some(AllocationReason::MediaLayoutDiscovery),
            },
        ));
        budget = budget.saturating_sub(playable.bytes.len());
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
        plan.allocations.push(allocation(
            snapshot,
            AllocationInputs {
                candidate: inputs.candidate,
                origin,
                playable,
                emergency: inputs.emergency,
                reason: None,
            },
        ));
        gained = gained.saturating_add(playable.playable_ms);
        budget = budget.saturating_sub(playable.bytes.len());
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

/// Repacks missing extents into origin request slices: contiguous
/// neighbours merge, and no slice exceeds the snapshot's request
/// bound. Gaps between extents are never bridged — a request must not
/// pay for bytes nothing asked for.
fn request_slices(missing: Vec<PlayableRange>, slice_bytes: u64) -> Vec<PlayableRange> {
    let slice_bytes = slice_bytes.max(1);
    let mut slices = Vec::new();
    let mut run: Option<PlayableRange> = None;
    for extent in missing {
        run = Some(match run {
            Some(open) if open.bytes.end == extent.bytes.start => PlayableRange {
                bytes: crate::ByteRange::new(open.bytes.start, extent.bytes.end),
                playable_ms: open.playable_ms.saturating_add(extent.playable_ms),
            },
            Some(open) => {
                push_run(&mut slices, open, slice_bytes);
                extent
            }
            None => extent,
        });
    }
    if let Some(open) = run {
        push_run(&mut slices, open, slice_bytes);
    }
    slices
}

fn push_run(slices: &mut Vec<PlayableRange>, run: PlayableRange, slice_bytes: u64) {
    let mut start = run.bytes.start;
    while start < run.bytes.end {
        let end = start.saturating_add(slice_bytes).min(run.bytes.end);
        let bytes = crate::ByteRange::new(start, end);
        slices.push(PlayableRange {
            bytes,
            playable_ms: proportional_gain(run, bytes.len()),
        });
        start = end;
    }
}

fn fit_to_budget(playable: PlayableRange, budget: u64) -> PlayableRange {
    if playable.bytes.len() <= budget {
        return playable;
    }
    let bytes = crate::ByteRange::new(playable.bytes.start, playable.bytes.start + budget);
    PlayableRange {
        bytes,
        playable_ms: proportional_gain(playable, budget),
    }
}

fn proportional_gain(playable: PlayableRange, bytes: u64) -> u64 {
    let scaled = u128::from(playable.playable_ms).saturating_mul(u128::from(bytes));
    (scaled / u128::from(playable.bytes.len().max(1)))
        .max(1)
        .min(u128::from(u64::MAX)) as u64
}

fn overlaps_planned(
    plan: &AllocationPlan,
    candidate: &CandidateSnapshot,
    range: crate::ByteRange,
) -> bool {
    plan.allocations.iter().any(|work| {
        work.post == candidate.post && work.range.start < range.end && range.start < work.range.end
    })
}

fn candidate_budget(snapshot: &PlayabilitySnapshot, inputs: &AppendInputs<'_>) -> u64 {
    if inputs.candidate.layout != MediaLayout::RequiresCompleteFile {
        return inputs.budget;
    }
    let missing_bytes = missing(inputs.candidate)
        .iter()
        .map(|playable| playable.bytes.len())
        .sum();
    inputs
        .budget
        .max(missing_bytes)
        .min(snapshot.storage.available_bytes())
}

pub(super) fn planned_bytes(plan: &AllocationPlan) -> u64 {
    plan.allocations
        .iter()
        .map(|allocation| allocation.range.len())
        .sum()
}
