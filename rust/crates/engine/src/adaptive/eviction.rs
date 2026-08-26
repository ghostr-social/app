use super::resources::storage_target_bytes;
use super::{CandidateSnapshot, Eviction, EvictionReason, PlayabilitySnapshot};
use crate::ByteRange;
use std::collections::HashSet;

mod geometry;
mod protection;
mod selection;

pub(super) fn evictions(snapshot: &PlayabilitySnapshot, current_need_bytes: u64) -> Vec<Eviction> {
    let targets = release_targets(snapshot, current_need_bytes);
    if targets.wanted() == 0 {
        return Vec::new();
    }
    let protected = protection::selected(snapshot);
    let mut candidates = eviction_candidates(snapshot, &protected);
    candidates.sort_by(eviction_order);
    selection::select(
        candidates,
        targets.wanted(),
        targets.hard,
        snapshot.storage.available_bytes(),
    )
}

pub(super) struct EvictionCandidate {
    eviction: Eviction,
    indivisible: bool,
    protected: bool,
    present_bytes: u64,
    physical_tail: bool,
}

fn eviction_candidates(
    snapshot: &PlayabilitySnapshot,
    protected: &HashSet<crate::PostId>,
) -> Vec<EvictionCandidate> {
    snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.post != snapshot.playback.current)
        .flat_map(|candidate| candidate_evictions(candidate, protected.contains(&candidate.post)))
        .collect()
}

fn candidate_evictions(candidate: &CandidateSnapshot, protect: bool) -> Vec<EvictionCandidate> {
    let Some(extents) = eviction_extents(candidate, protect) else {
        return Vec::new();
    };
    let present = crate::media_timeline::normalize(candidate.present.clone());
    let present_bytes = present.iter().map(ByteRange::len).sum();
    let physical_end = present.last().map(|range| range.end);
    extents
        .iter()
        .copied()
        .map(|extent| EvictionCandidate {
            eviction: Eviction {
                post: candidate.post.clone(),
                range: extent.range,
                expected_playable_loss_ms: playable_loss(candidate, extent.range),
                reason: EvictionReason::StoragePressure,
            },
            indivisible: candidate.finalized,
            protected: extent.protected,
            present_bytes,
            physical_tail: physical_end == Some(extent.range.end),
        })
        .collect()
}

fn eviction_extents(candidate: &CandidateSnapshot, protect: bool) -> Option<Vec<geometry::Extent>> {
    if !candidate.finalized {
        return Some(geometry::partition(
            &candidate.present,
            protected_ranges(candidate, protect),
        ));
    }
    let total = candidate.total_bytes.filter(|total| *total > 0)?;
    let present = crate::media_timeline::normalize(candidate.present.clone());
    (present == [ByteRange::new(0, total)]).then(|| {
        vec![geometry::Extent {
            range: present[0],
            protected: !protected_ranges(candidate, protect).is_empty(),
        }]
    })
}

fn protected_ranges(candidate: &CandidateSnapshot, protect: bool) -> &[ByteRange] {
    if !protect {
        return &[];
    }
    candidate
        .startup
        .as_ref()
        .map_or(&[], |startup| startup.ranges())
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

fn eviction_order(left: &EvictionCandidate, right: &EvictionCandidate) -> core::cmp::Ordering {
    density(left)
        .total_cmp(&density(right))
        .then_with(|| right.eviction.post.cmp(&left.eviction.post))
        .then_with(|| right.eviction.range.start.cmp(&left.eviction.range.start))
}

fn density(candidate: &EvictionCandidate) -> f64 {
    candidate.eviction.expected_playable_loss_ms / candidate.eviction.range.len().max(1) as f64
}

/// Releases whatever the soft target demands, and additionally enough
/// hard room for the bytes the current video will write this pass:
/// writing into a full store makes it reject or sweep wholesale,
/// forfeiting paid sibling bytes that then have to be bought again.
#[derive(Clone, Copy)]
struct ReleaseTargets {
    soft: u64,
    hard: u64,
}

impl ReleaseTargets {
    fn wanted(self) -> u64 {
        self.soft.max(self.hard)
    }
}

fn release_targets(snapshot: &PlayabilitySnapshot, current_need_bytes: u64) -> ReleaseTargets {
    let soft = snapshot
        .storage
        .used_bytes
        .saturating_sub(storage_target_bytes(snapshot));
    let hard = snapshot
        .storage
        .used_bytes
        .saturating_add(current_need_bytes)
        .saturating_sub(snapshot.storage.budget_bytes);
    ReleaseTargets { soft, hard }
}
