use super::ranges::missing;
use super::sources::delivery_ms;
use super::{CandidateSnapshot, MediaLayout, OriginHealth, PlayabilitySnapshot};

pub(super) fn admitted(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    origin: &OriginHealth,
) -> bool {
    if !candidate.retrieval_eligible || candidate.layout == MediaLayout::Unknown {
        return false;
    }
    if candidate.post == snapshot.playback.current || candidate.layout == MediaLayout::Streamable {
        return true;
    }
    complete_file_fits_storage(snapshot, candidate)
        && complete_file_delivery_ms(snapshot, candidate, origin)
            <= coverage_before(snapshot, candidate)
}

fn complete_file_delivery_ms(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    origin: &OriginHealth,
) -> u64 {
    let bytes = missing(candidate)
        .iter()
        .map(|playable| playable.bytes.len())
        .sum();
    delivery_ms(snapshot, origin, bytes)
}

fn complete_file_fits_storage(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
) -> bool {
    missing(candidate)
        .iter()
        .map(|playable| playable.bytes.len())
        .sum::<u64>()
        <= snapshot.storage.available_bytes()
}

fn coverage_before(snapshot: &PlayabilitySnapshot, candidate: &CandidateSnapshot) -> u64 {
    let cached = snapshot
        .candidates
        .iter()
        .filter(|other| other.retrieval_eligible)
        .filter(|other| other.feed_offset.magnitude() < candidate.feed_offset.magnitude())
        .filter(|other| other.post != snapshot.playback.current)
        .map(cached_playable_ms)
        .sum::<u64>();
    snapshot.playback.buffer_ahead_ms.saturating_add(cached)
}

fn cached_playable_ms(candidate: &CandidateSnapshot) -> u64 {
    if !super::reserve_model::is_structural(candidate) {
        return 0;
    }
    candidate
        .playable_ranges
        .iter()
        .filter(|playable| covered(playable.bytes, &candidate.present))
        .map(|playable| playable.playable_ms)
        .sum()
}

fn covered(wanted: crate::ByteRange, present: &[crate::ByteRange]) -> bool {
    present
        .iter()
        .any(|range| range.start <= wanted.start && range.end >= wanted.end)
}
