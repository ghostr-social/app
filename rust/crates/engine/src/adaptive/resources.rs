use super::ranges::uncovered_bytes;
use super::{CandidateSnapshot, PlayabilitySnapshot, PlaybackSnapshot};
use crate::playback::{EstimateConfidence, PlaybackPhase};

const STORAGE_TARGET_BPS: u64 = 9_900;

/// An emergency exists only while network work can still improve the
/// current video. A fully stored current post (including the natural
/// low-buffer tail of every finished download) must not displace the
/// speculative work the next swipe depends on. In-flight ranges do
/// not settle a post: work still on the wire keeps its lane priority.
pub(super) fn endangered(snapshot: &PlayabilitySnapshot, current: &CandidateSnapshot) -> bool {
    if fully_stored(current) {
        return false;
    }
    phase_endangered(&snapshot.playback)
        || snapshot.playback.buffer_ahead_ms < 4_000
        || effective_network_bps(snapshot) < current.bitrate_bps
}

pub(super) fn fully_stored(candidate: &CandidateSnapshot) -> bool {
    candidate
        .playable_ranges
        .iter()
        .all(|playable| uncovered_bytes(playable.bytes, &candidate.present) == 0)
}

pub(super) fn speculative_budget(snapshot: &PlayabilitySnapshot) -> u64 {
    if storage_displaces_speculation(snapshot) {
        return 0;
    }
    let horizon_ms = speculation_horizon_ms(snapshot);
    let network = effective_network_bps(snapshot).saturating_mul(horizon_ms) / 8_000;
    let weighted = network
        .saturating_mul(connection_weight_bps(snapshot))
        .saturating_mul(rtt_weight_bps(snapshot))
        / 100_000_000;
    weighted.min(speculative_storage_bytes(snapshot))
}

pub(super) fn upcoming_depth_ms(snapshot: &PlayabilitySnapshot) -> u64 {
    match snapshot.navigation.forward_swipes_per_minute >= 12 {
        true => 2_000,
        false => 6_000,
    }
}

pub(super) fn effective_network_bps(snapshot: &PlayabilitySnapshot) -> u64 {
    let loss = u64::from(snapshot.network.packet_loss_bps.min(10_000));
    snapshot
        .network
        .throughput_bps
        .saturating_mul(10_000 - loss)
        .saturating_mul(confidence_bps(snapshot.network.confidence))
        / 100_000_000
}

pub(super) fn storage_displaces_speculation(snapshot: &PlayabilitySnapshot) -> bool {
    snapshot.storage.used_bytes >= storage_target_bytes(snapshot)
}

pub(super) fn storage_target_bytes(snapshot: &PlayabilitySnapshot) -> u64 {
    snapshot
        .storage
        .budget_bytes
        .saturating_mul(STORAGE_TARGET_BPS)
        / 10_000
}

fn speculative_storage_bytes(snapshot: &PlayabilitySnapshot) -> u64 {
    storage_target_bytes(snapshot)
        .saturating_sub(snapshot.storage.used_bytes)
        .saturating_sub(reserved_storage_bytes(snapshot))
}

fn reserved_storage_bytes(snapshot: &PlayabilitySnapshot) -> u64 {
    snapshot
        .candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .in_flight
                .iter()
                .filter(|active| active.identity_current)
                .map(|active| uncovered_bytes(active.bytes, &candidate.present))
        })
        .sum()
}

fn phase_endangered(playback: &PlaybackSnapshot) -> bool {
    matches!(
        playback.phase,
        PlaybackPhase::Starting | PlaybackPhase::NetworkStalled
    )
}

fn speculation_horizon_ms(snapshot: &PlayabilitySnapshot) -> u64 {
    let rate = u64::from(snapshot.navigation.forward_swipes_per_minute);
    4_000_u64.saturating_add(rate.saturating_mul(100))
}

fn connection_weight_bps(snapshot: &PlayabilitySnapshot) -> u64 {
    let capacity = snapshot.network.connection_capacity.min(6) as u64;
    2_500_u64.saturating_add(capacity.saturating_mul(1_250))
}

fn rtt_weight_bps(snapshot: &PlayabilitySnapshot) -> u64 {
    10_000_000_u64 / 1_000_u64.saturating_add(snapshot.network.rtt_ms)
}

fn confidence_bps(confidence: EstimateConfidence) -> u64 {
    match confidence {
        EstimateConfidence::Low => 6_000,
        EstimateConfidence::Medium => 8_000,
        EstimateConfidence::High => 10_000,
    }
}
