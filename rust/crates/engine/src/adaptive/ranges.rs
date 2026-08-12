use super::{CandidateSnapshot, PlayableRange};
use crate::ByteRange;

const EVICTION_REACQUIRE_PROBABILITY: f64 = 0.75;

pub(super) fn missing(candidate: &CandidateSnapshot) -> Vec<PlayableRange> {
    let blocked = blocked_ranges(candidate);
    candidate
        .playable_ranges
        .iter()
        .flat_map(|playable| subtract(*playable, &blocked))
        .collect()
}

pub(super) fn missing_playable(
    candidate: &CandidateSnapshot,
    playable: PlayableRange,
) -> Vec<PlayableRange> {
    subtract(playable, &blocked_ranges(candidate))
}

pub(super) fn playable_gain(candidate: &CandidateSnapshot, bytes: ByteRange) -> u64 {
    candidate
        .playable_ranges
        .iter()
        .filter_map(|playable| {
            let start = playable.bytes.start.max(bytes.start);
            let end = playable.bytes.end.min(bytes.end);
            (start < end).then(|| proportional_gain(*playable, end - start))
        })
        .sum()
}

pub(super) fn uncovered_bytes(bytes: ByteRange, present: &[ByteRange]) -> u64 {
    let covered: u64 = crate::media_timeline::normalize(present.to_vec())
        .iter()
        .map(|range| {
            let start = range.start.max(bytes.start);
            let end = range.end.min(bytes.end);
            end.saturating_sub(start)
        })
        .sum();
    bytes.len().saturating_sub(covered)
}

fn blocked_ranges(candidate: &CandidateSnapshot) -> Vec<ByteRange> {
    let mut blocked = candidate.present.clone();
    if candidate.view_probability.value() < EVICTION_REACQUIRE_PROBABILITY {
        blocked.extend(candidate.recently_evicted.iter().copied());
    }
    blocked.extend(
        candidate
            .in_flight
            .iter()
            .filter(|active| active.identity_current)
            .map(|active| active.bytes),
    );
    crate::media_timeline::normalize(blocked)
}

fn subtract(playable: PlayableRange, blocked: &[ByteRange]) -> Vec<PlayableRange> {
    let mut cursor = playable.bytes.start;
    let mut missing = Vec::new();
    for range in blocked
        .iter()
        .filter(|range| overlaps(**range, playable.bytes))
    {
        push_gap(&mut missing, playable, cursor, range.start);
        cursor = cursor.max(range.end).min(playable.bytes.end);
    }
    push_gap(&mut missing, playable, cursor, playable.bytes.end);
    missing
}

fn push_gap(missing: &mut Vec<PlayableRange>, playable: PlayableRange, start: u64, end: u64) {
    let start = start.max(playable.bytes.start);
    let end = end.min(playable.bytes.end);
    if start >= end {
        return;
    }
    let bytes = ByteRange::new(start, end);
    let gain = proportional_gain(playable, bytes.len());
    missing.push(PlayableRange {
        bytes,
        playable_ms: gain,
    });
}

fn proportional_gain(playable: PlayableRange, missing_bytes: u64) -> u64 {
    let numerator = u128::from(playable.playable_ms).saturating_mul(u128::from(missing_bytes));
    (numerator / u128::from(playable.bytes.len().max(1)))
        .max(1)
        .min(u128::from(u64::MAX)) as u64
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
