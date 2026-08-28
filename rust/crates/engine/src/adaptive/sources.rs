use super::ranges::missing;
use super::{CandidateSnapshot, OriginHealth, PlayabilitySnapshot};
use crate::RequestAuthority;

pub(super) fn best_origin(candidate: &CandidateSnapshot) -> Option<&OriginHealth> {
    best_origin_where(candidate, |_| true)
}

pub(super) fn best_origin_where<'a>(
    candidate: &'a CandidateSnapshot,
    mut admitted: impl FnMut(&OriginHealth) -> bool,
) -> Option<&'a OriginHealth> {
    if let Some(preferred) = candidate.preferred_source.as_deref() {
        if let Some(origin) = candidate
            .origins
            .iter()
            .find(|origin| retrievable(origin) && admitted(origin) && origin.source == preferred)
        {
            return Some(origin);
        }
    }
    candidate
        .origins
        .iter()
        .filter(|origin| retrievable(origin) && admitted(origin))
        .reduce(faster)
}

pub(super) fn best_available(origins: &[OriginHealth]) -> Option<&OriginHealth> {
    origins
        .iter()
        .filter(|origin| retrievable(origin))
        .reduce(faster)
}

fn faster<'a>(best: &'a OriginHealth, candidate: &'a OriginHealth) -> &'a OriginHealth {
    if effective_throughput(candidate) > effective_throughput(best) {
        candidate
    } else {
        best
    }
}

pub(in crate::adaptive) fn retrievable(origin: &OriginHealth) -> bool {
    origin.available && RequestAuthority::from_url(&origin.source).is_some()
}

pub(super) fn candidate_score(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
) -> f64 {
    let Some(origin) = best_origin(candidate) else {
        return 0.0;
    };
    let Some(playable) = missing(candidate).first().copied() else {
        return 0.0;
    };
    let cost = delivery_ms(snapshot, origin, playable.bytes.len()).max(1);
    candidate.view_probability.value() * playable.playable_ms as f64 / cost as f64
}

pub(super) fn delivery_ms(
    snapshot: &PlayabilitySnapshot,
    origin: &OriginHealth,
    bytes: u64,
) -> u64 {
    let throughput = effective_throughput(origin)
        .min(super::resources::effective_network_bps(snapshot))
        .max(1);
    bytes.saturating_mul(8_000).div_ceil(throughput) + origin.rtt_ms.max(snapshot.network.rtt_ms)
}

fn effective_throughput(origin: &OriginHealth) -> u64 {
    let loss = u64::from(origin.packet_loss_bps.min(10_000));
    let failure = u64::from(origin.failure_bps.min(10_000));
    origin
        .throughput_bps
        .saturating_mul(10_000 - loss)
        .saturating_mul(10_000 - failure)
        / 100_000_000
}
