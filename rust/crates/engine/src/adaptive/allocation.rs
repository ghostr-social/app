use super::admission::admitted;
use super::allocation_evidence::{allocation, AllocationInputs};
use super::plan::{AllocationPlan, AllocationReason};
use super::ranges::{missing, missing_playable};
use super::sources::best_origin;
use super::{CandidateSnapshot, MediaLayout, OriginHealth, PlayabilitySnapshot, PlayableRange};

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
    for playable in missing_playable(inputs.candidate, probe) {
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
    for available in missing(inputs.candidate) {
        if gained >= inputs.target_ms || budget == 0 {
            break;
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
