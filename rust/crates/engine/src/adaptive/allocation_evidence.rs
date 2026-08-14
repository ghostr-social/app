use super::plan::{Allocation, AllocationReason, CandidateUtility, PreemptionAuthority};
use super::sources::delivery_ms;
use super::{CandidateSnapshot, OriginHealth, PlayabilitySnapshot, PlayableRange};

pub(super) struct AllocationInputs<'a> {
    pub(super) candidate: &'a CandidateSnapshot,
    pub(super) origin: &'a OriginHealth,
    pub(super) playable: PlayableRange,
    pub(super) emergency: bool,
    pub(super) reason: Option<AllocationReason>,
}

pub(super) fn allocation(
    snapshot: &PlayabilitySnapshot,
    inputs: AllocationInputs<'_>,
) -> Allocation {
    let utility = candidate_utility(snapshot, inputs.candidate, inputs.origin, inputs.playable);
    Allocation {
        post: inputs.candidate.post.clone(),
        range: inputs.playable.bytes,
        source: inputs.origin.source.clone(),
        expected_playable_gain_ms: inputs.playable.playable_ms,
        utility,
        authority: authority(inputs.candidate, snapshot, inputs.emergency),
        commitment_until_ms: commitment_until(snapshot, utility.expected_delivery_ms),
        reason: inputs
            .reason
            .unwrap_or_else(|| reason(inputs.candidate, snapshot, inputs.emergency)),
    }
}

fn commitment_until(snapshot: &PlayabilitySnapshot, delivery_ms: u64) -> u64 {
    let shared = delivery_ms.saturating_mul(snapshot.network.connection_ceiling as u64);
    snapshot
        .observed_at_ms
        .saturating_add(snapshot.commitment_ms)
        .saturating_add(shared)
}

pub(super) fn candidate_utility(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    origin: &OriginHealth,
    playable: PlayableRange,
) -> CandidateUtility {
    let expected = delivery_ms(snapshot, origin, playable.bytes.len());
    let probability = candidate.view_probability.value();
    CandidateUtility {
        view_probability: probability,
        additional_playable_ms: playable.playable_ms,
        expected_delivery_ms: expected,
        score: probability * playable.playable_ms as f64 / expected.max(1) as f64,
    }
}

fn authority(
    candidate: &CandidateSnapshot,
    snapshot: &PlayabilitySnapshot,
    emergency: bool,
) -> PreemptionAuthority {
    if candidate.post == snapshot.playback.current
        && snapshot.playback.authority == super::CurrentAuthority::Provisional
    {
        return PreemptionAuthority::Speculative;
    }
    if candidate.post == snapshot.playback.current && emergency {
        return PreemptionAuthority::PlaybackCritical;
    }
    match candidate.feed_offset.magnitude() <= 1 {
        true => PreemptionAuthority::Transition,
        false => PreemptionAuthority::Speculative,
    }
}

fn reason(
    candidate: &CandidateSnapshot,
    snapshot: &PlayabilitySnapshot,
    emergency: bool,
) -> AllocationReason {
    if candidate.post == snapshot.playback.current {
        return match emergency {
            true => AllocationReason::CurrentStallPrevention,
            false => AllocationReason::CurrentBufferReserve,
        };
    }
    match snapshot.navigation.forward_swipes_per_minute >= 12 {
        true => AllocationReason::RapidNavigationCoverage,
        false => AllocationReason::LikelyNextTransition,
    }
}
