use crate::adaptive::{ActionKind, AllocationPlan, CandidateSnapshot, MediaLayout};
use crate::ByteRange;

pub(super) fn gain(
    candidate: &CandidateSnapshot,
    action: &ActionKind,
    base: &AllocationPlan,
) -> u64 {
    match action {
        ActionKind::FetchWhole { maximum_bytes }
            if candidate
                .total_bytes
                .is_some_and(|total| *maximum_bytes >= total) =>
        {
            candidate.duration_ms
        }
        ActionKind::Prefix(range) | ActionKind::FetchRange(range)
            if candidate.layout != MediaLayout::Unknown =>
        {
            marginal(candidate, *range, base)
        }
        _ => 0,
    }
}

fn marginal(candidate: &CandidateSnapshot, range: ByteRange, base: &AllocationPlan) -> u64 {
    let before = conditional_coverage(candidate, base);
    let mut after = before.clone();
    after.push(range);
    candidate
        .playable_ranges
        .iter()
        .filter(|item| overlaps(item.bytes, range))
        .filter(|item| crate::adaptive::ranges::uncovered_bytes(item.bytes, &before) > 0)
        .filter(|item| crate::adaptive::ranges::uncovered_bytes(item.bytes, &after) == 0)
        .map(|item| item.playable_ms)
        .sum()
}

fn conditional_coverage(candidate: &CandidateSnapshot, base: &AllocationPlan) -> Vec<ByteRange> {
    let mut coverage = candidate.present.clone();
    if candidate.layout != MediaLayout::Unknown {
        coverage.extend(
            base.retained
                .iter()
                .filter(|work| work.post == candidate.post)
                .map(|work| work.request.requested_bytes()),
        );
    }
    coverage
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
