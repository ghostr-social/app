use super::{AllocationPlan, CandidateSnapshot, PlayableRange};

pub(super) fn request_slices(missing: Vec<PlayableRange>, slice_bytes: u64) -> Vec<PlayableRange> {
    let slice_bytes = slice_bytes.max(1);
    let mut slices = Vec::new();
    let mut run: Option<PlayableRange> = None;
    for extent in missing {
        run = Some(join_or_flush(&mut slices, run, extent, slice_bytes));
    }
    if let Some(open) = run {
        push_run(&mut slices, open, slice_bytes);
    }
    slices
}

fn join_or_flush(
    slices: &mut Vec<PlayableRange>,
    open: Option<PlayableRange>,
    extent: PlayableRange,
    slice_bytes: u64,
) -> PlayableRange {
    match open {
        Some(open) if open.bytes.end == extent.bytes.start => PlayableRange {
            bytes: crate::ByteRange::new(open.bytes.start, extent.bytes.end),
            playable_ms: open.playable_ms.saturating_add(extent.playable_ms),
        },
        Some(open) => {
            push_run(slices, open, slice_bytes);
            extent
        }
        None => extent,
    }
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

pub(super) fn fit_to_budget(playable: PlayableRange, budget: u64) -> PlayableRange {
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

pub(super) fn overlaps_planned(
    plan: &AllocationPlan,
    candidate: &CandidateSnapshot,
    range: crate::ByteRange,
) -> bool {
    plan.allocations.iter().any(|work| {
        let reserved = work.request.reserved_coverage();
        work.post == candidate.post && reserved.start < range.end && range.start < reserved.end
    })
}
