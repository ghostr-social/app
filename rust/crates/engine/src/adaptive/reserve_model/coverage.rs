use super::super::reserve_candidate::ReserveCandidate;
use super::super::CandidateSnapshot;

pub(super) fn ready(candidates: &[ReserveCandidate<'_>], horizon_ms: u64) -> u64 {
    candidates
        .iter()
        .filter_map(|candidate| candidate.progressive())
        .map(|candidate| weighted(candidate, horizon_ms))
        .sum()
}

fn weighted(candidate: &CandidateSnapshot, horizon_ms: u64) -> u64 {
    if !super::is_ready(candidate) {
        return 0;
    }
    let playable = contiguous_playable_ms(candidate).min(horizon_ms);
    (playable as f64 * candidate.view_probability.value()).floor() as u64
}

fn contiguous_playable_ms(candidate: &CandidateSnapshot) -> u64 {
    candidate
        .playable_ranges
        .iter()
        .take_while(|playable| super::uncovered_bytes(playable.bytes, &candidate.present) == 0)
        .map(|playable| playable.playable_ms)
        .sum()
}
