use super::resources::{storage_displaces_speculation, storage_target_bytes};
use super::{CandidateSnapshot, Eviction, EvictionReason, PlayabilitySnapshot};
use crate::ByteRange;

pub(super) fn evictions(snapshot: &PlayabilitySnapshot) -> Vec<Eviction> {
    if !storage_displaces_speculation(snapshot) {
        return Vec::new();
    }
    let mut candidates = eviction_candidates(snapshot);
    candidates.sort_by(eviction_order);
    take_until(candidates, bytes_to_release(snapshot))
}

fn eviction_candidates(snapshot: &PlayabilitySnapshot) -> Vec<Eviction> {
    snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.post != snapshot.playback.current)
        .flat_map(candidate_evictions)
        .collect()
}

fn candidate_evictions(candidate: &CandidateSnapshot) -> Vec<Eviction> {
    candidate
        .present
        .iter()
        .copied()
        .map(|range| Eviction {
            post: candidate.post.clone(),
            range,
            expected_playable_loss_ms: playable_loss(candidate, range),
            reason: EvictionReason::StoragePressure,
        })
        .collect()
}

fn playable_loss(candidate: &CandidateSnapshot, stored: ByteRange) -> f64 {
    let milliseconds: u64 = candidate
        .playable_ranges
        .iter()
        .map(|playable| overlap_gain(*playable, stored))
        .sum();
    candidate.view_probability.value() * milliseconds as f64
}

fn overlap_gain(playable: super::PlayableRange, stored: ByteRange) -> u64 {
    let start = playable.bytes.start.max(stored.start);
    let end = playable.bytes.end.min(stored.end);
    let overlap = end.saturating_sub(start);
    let scaled = u128::from(playable.playable_ms).saturating_mul(u128::from(overlap));
    (scaled / u128::from(playable.bytes.len().max(1))) as u64
}

fn eviction_order(left: &Eviction, right: &Eviction) -> std::cmp::Ordering {
    left.expected_playable_loss_ms
        .total_cmp(&right.expected_playable_loss_ms)
        .then_with(|| right.post.cmp(&left.post))
        .then_with(|| right.range.start.cmp(&left.range.start))
}

fn bytes_to_release(snapshot: &PlayabilitySnapshot) -> u64 {
    snapshot
        .storage
        .used_bytes
        .saturating_sub(storage_target_bytes(snapshot))
}

fn take_until(candidates: Vec<Eviction>, wanted: u64) -> Vec<Eviction> {
    let mut remaining = wanted;
    let mut selected = Vec::new();
    for candidate in candidates {
        if remaining == 0 {
            break;
        }
        let eviction = exact_tail(candidate, remaining);
        remaining = remaining.saturating_sub(eviction.range.len());
        selected.push(eviction);
    }
    selected
}

fn exact_tail(mut eviction: Eviction, maximum: u64) -> Eviction {
    let original = eviction.range.len();
    if original <= maximum {
        return eviction;
    }
    eviction.range.start = eviction.range.end - maximum;
    eviction.expected_playable_loss_ms *= maximum as f64 / original as f64;
    eviction
}
