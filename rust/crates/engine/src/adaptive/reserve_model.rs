use super::ranges::uncovered_bytes;
use super::sources::{best_origin, delivery_ms};
use super::{CandidateSnapshot, MediaLayout, PlayabilitySnapshot};
use crate::playback::EstimateConfidence;

const MAX_READY_VIDEOS: usize = 5;
const RESERVE_RISK_BPS: u16 = 500;
const FORWARD_SWIPE_PRIOR_PER_MINUTE: u64 = 4;
const MIN_RECOVERY_HORIZON_MS: u64 = 250;
const MAX_RECOVERY_HORIZON_MS: u64 = 30_000;

pub(super) struct ReserveTarget {
    pub(super) count: usize,
    pub(super) recovery_horizon_ms: u64,
    pub(super) underflow_risk_bps: u16,
}

pub(super) fn candidates(snapshot: &PlayabilitySnapshot) -> Vec<&CandidateSnapshot> {
    let mut candidates: Vec<_> = snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.feed_offset.value() > 0)
        .collect();
    candidates.sort_by_key(|candidate| candidate.feed_offset.magnitude());
    candidates.truncate(MAX_READY_VIDEOS);
    candidates
}

pub(super) fn target(
    snapshot: &PlayabilitySnapshot,
    candidates: &[&CandidateSnapshot],
) -> ReserveTarget {
    let recovery_horizon_ms = recovery_horizon(snapshot, candidates);
    let lambda = swipe_lambda(snapshot, recovery_horizon_ms);
    let count = target_count(lambda, candidates.len());
    ReserveTarget {
        count,
        recovery_horizon_ms,
        underflow_risk_bps: poisson_tail_bps(lambda, count),
    }
}

pub(super) fn is_ready(candidate: &CandidateSnapshot) -> bool {
    candidate.layout != MediaLayout::Unknown
        && readiness_ranges(candidate)
            .iter()
            .all(|range| uncovered_bytes(*range, &candidate.present) == 0)
}

pub(super) fn is_in_flight(candidate: &CandidateSnapshot) -> bool {
    if candidate.layout == MediaLayout::Unknown || is_ready(candidate) {
        return false;
    }
    let mut covered = candidate.present.clone();
    covered.extend(
        candidate
            .in_flight
            .iter()
            .filter(|active| active.identity_current)
            .map(|active| active.bytes),
    );
    readiness_ranges(candidate)
        .iter()
        .all(|range| uncovered_bytes(*range, &covered) == 0)
}

pub(super) fn readiness_ranges(candidate: &CandidateSnapshot) -> Vec<crate::ByteRange> {
    let mut ranges = Vec::new();
    if let Some(probe) = candidate.timeline_probe {
        ranges.push(probe.bytes);
    }
    if let Some(first) = candidate.playable_ranges.first() {
        if !ranges.contains(&first.bytes) {
            ranges.push(first.bytes);
        }
    }
    ranges
}

pub(super) fn ready_coverage_ms(candidates: &[&CandidateSnapshot], horizon_ms: u64) -> u64 {
    candidates
        .iter()
        .map(|candidate| weighted_coverage(candidate, horizon_ms))
        .sum()
}

fn recovery_horizon(snapshot: &PlayabilitySnapshot, candidates: &[&CandidateSnapshot]) -> u64 {
    candidates
        .iter()
        .filter(|candidate| !is_ready(candidate))
        .filter_map(|candidate| candidate_recovery(snapshot, candidate))
        .min()
        .unwrap_or(MIN_RECOVERY_HORIZON_MS)
        .clamp(MIN_RECOVERY_HORIZON_MS, MAX_RECOVERY_HORIZON_MS)
}

fn candidate_recovery(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
) -> Option<u64> {
    let origin = best_origin(candidate)?;
    let bytes: u64 = readiness_ranges(candidate)
        .iter()
        .map(|range| uncovered_bytes(*range, &candidate.present))
        .sum();
    let base = delivery_ms(snapshot, origin, bytes.max(1));
    Some(base.saturating_mul(risk_weight(snapshot, candidate)) / 10_000)
}

fn risk_weight(snapshot: &PlayabilitySnapshot, candidate: &CandidateSnapshot) -> u64 {
    let confidence: u64 = match snapshot.network.confidence {
        EstimateConfidence::High => 12_500,
        EstimateConfidence::Medium => 15_000,
        EstimateConfidence::Low => 20_000,
    };
    match candidate.layout {
        MediaLayout::Unknown => confidence.saturating_mul(2),
        _ => confidence,
    }
}

fn swipe_lambda(snapshot: &PlayabilitySnapshot, horizon_ms: u64) -> f64 {
    let observed = u64::from(snapshot.navigation.forward_swipes_per_minute);
    (observed + FORWARD_SWIPE_PRIOR_PER_MINUTE) as f64 * horizon_ms as f64 / 60_000.0
}

fn target_count(lambda: f64, available: usize) -> usize {
    if available == 0 {
        return 0;
    }
    (1..=available)
        .find(|count| poisson_tail_bps(lambda, *count) <= RESERVE_RISK_BPS)
        .unwrap_or(available)
}

fn poisson_tail_bps(lambda: f64, count: usize) -> u16 {
    let mut term = (-lambda).exp();
    let mut cumulative = term;
    for value in 1..=count {
        term *= lambda / value as f64;
        cumulative += term;
    }
    ((1.0 - cumulative).clamp(0.0, 1.0) * 10_000.0).ceil() as u16
}

fn weighted_coverage(candidate: &CandidateSnapshot, horizon_ms: u64) -> u64 {
    if !is_ready(candidate) {
        return 0;
    }
    let playable = contiguous_playable_ms(candidate).min(horizon_ms);
    (playable as f64 * candidate.view_probability.value()).floor() as u64
}

fn contiguous_playable_ms(candidate: &CandidateSnapshot) -> u64 {
    candidate
        .playable_ranges
        .iter()
        .take_while(|playable| uncovered_bytes(playable.bytes, &candidate.present) == 0)
        .map(|playable| playable.playable_ms)
        .sum()
}
