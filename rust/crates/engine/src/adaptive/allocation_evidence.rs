use super::plan::{Allocation, AllocationReason, CandidateUtility, PreemptionAuthority};
use super::sources::delivery_ms;
use super::{
    CandidateSnapshot, OriginHealth, PlayabilitySnapshot, PlayableRange, PromotionGrant,
    RetrievalRequest, WholeBodyContract, WholeFetchReason,
};

const COLD_DIRECT_FETCH_CAP_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy)]
pub(super) struct AllocationInputs<'a> {
    pub(super) candidate: &'a CandidateSnapshot,
    pub(super) origin: &'a OriginHealth,
    pub(super) playable: PlayableRange,
    pub(super) emergency: bool,
    pub(super) reason: Option<AllocationReason>,
    pub(super) reservation_budget: u64,
}

pub(super) fn allocation(
    snapshot: &PlayabilitySnapshot,
    inputs: AllocationInputs<'_>,
) -> Allocation {
    let utility = candidate_utility(snapshot, inputs.candidate, inputs.origin, inputs.playable);
    let promotion = promotion_grant(snapshot, &inputs);
    let request = RetrievalRequest::FetchRange {
        bytes: inputs.playable.bytes,
        promotion,
    };
    let delivery = delivery_ms(snapshot, inputs.origin, request.reserved_network_bytes());
    Allocation {
        post: inputs.candidate.post.clone(),
        request,
        source: inputs.origin.source.clone(),
        expected_playable_gain_ms: inputs.playable.playable_ms,
        utility,
        authority: authority(inputs.candidate, snapshot, inputs.emergency),
        commitment_until_ms: commitment_until(snapshot, delivery),
        reason: inputs
            .reason
            .unwrap_or_else(|| reason(inputs.candidate, snapshot, inputs.emergency)),
    }
}

pub(super) fn whole_allocation(
    snapshot: &PlayabilitySnapshot,
    inputs: AllocationInputs<'_>,
) -> Allocation {
    let utility = candidate_utility(snapshot, inputs.candidate, inputs.origin, inputs.playable);
    let request = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped {
            maximum_bytes: inputs.playable.bytes.len(),
        },
        reason: WholeFetchReason::PlannedCompletion,
    };
    Allocation {
        post: inputs.candidate.post.clone(),
        request,
        source: inputs.origin.source.clone(),
        expected_playable_gain_ms: inputs.playable.playable_ms,
        utility,
        authority: authority(inputs.candidate, snapshot, inputs.emergency),
        commitment_until_ms: commitment_until(
            snapshot,
            delivery_ms(snapshot, inputs.origin, request.reserved_network_bytes()),
        ),
        reason: inputs
            .reason
            .unwrap_or_else(|| reason(inputs.candidate, snapshot, inputs.emergency)),
    }
}

fn promotion_grant(
    snapshot: &PlayabilitySnapshot,
    inputs: &AllocationInputs<'_>,
) -> Option<PromotionGrant> {
    let total = inputs.candidate.total_bytes?;
    let delivery = delivery_ms(snapshot, inputs.origin, total);
    let on_time = delivery <= snapshot.commitment_ms;
    let fits = total <= inputs.reservation_budget;
    (inputs.candidate.evidence.size.reliable
        && total <= COLD_DIRECT_FETCH_CAP_BYTES
        && on_time
        && fits)
        .then(|| PromotionGrant {
            maximum_bytes: total,
            valid_until_ms: commitment_until(snapshot, delivery),
        })
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
    if candidate.feed_offset.magnitude() <= 1 {
        PreemptionAuthority::Transition
    } else {
        PreemptionAuthority::Speculative
    }
}

fn reason(
    candidate: &CandidateSnapshot,
    snapshot: &PlayabilitySnapshot,
    emergency: bool,
) -> AllocationReason {
    if candidate.post == snapshot.playback.current {
        return if emergency {
            AllocationReason::CurrentStallPrevention
        } else {
            AllocationReason::CurrentBufferReserve
        };
    }
    if snapshot.navigation.forward_swipes_per_minute >= 12 {
        AllocationReason::RapidNavigationCoverage
    } else {
        AllocationReason::LikelyNextTransition
    }
}
