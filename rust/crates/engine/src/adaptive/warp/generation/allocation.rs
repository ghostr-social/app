use super::builder::Builder;
use crate::adaptive::{
    Allocation, AllocationReason, CandidateSnapshot, CandidateUtility, ControlMode,
    PlayabilitySnapshot, PreemptionAuthority, RetrievalRequest, WholeBodyContract,
    WholeFetchReason,
};
use crate::ByteRange;

pub(super) struct AllocationSpec<'a> {
    request: RetrievalRequest,
    source: &'a str,
    playable_ms: u64,
    reason: AllocationReason,
}

impl<'a> AllocationSpec<'a> {
    pub(super) fn range(bytes: ByteRange, source: &'a str, playable_ms: u64) -> Self {
        Self {
            request: RetrievalRequest::FetchRange {
                bytes,
                promotion: None,
            },
            source,
            playable_ms,
            reason: AllocationReason::MediaLayoutDiscovery,
        }
    }

    pub(super) fn cache(bytes: ByteRange, source: &'a str) -> Self {
        Self {
            reason: AllocationReason::UsefulCommitment,
            ..Self::range(bytes, source, 0)
        }
    }

    pub(super) fn whole(bytes: u64, source: &'a str, playable_ms: u64) -> Self {
        Self {
            request: RetrievalRequest::FetchWhole {
                contract: WholeBodyContract::Capped {
                    maximum_bytes: bytes,
                },
                reason: WholeFetchReason::DirectCrossover,
            },
            source,
            playable_ms,
            reason: AllocationReason::NextStartability,
        }
    }

    pub(super) fn hedge(request: RetrievalRequest, source: &'a str) -> Self {
        Self {
            request,
            source,
            playable_ms: 0,
            reason: AllocationReason::CurrentStallPrevention,
        }
    }
}

impl Builder<'_> {
    pub(super) fn allocation(
        &self,
        candidate: &CandidateSnapshot,
        spec: AllocationSpec<'_>,
    ) -> Allocation {
        let expected = self
            .prediction(candidate, &kind_for_request(spec.request), spec.source)
            .forecast
            .completion
            .expected_ms;
        let allocation = Allocation {
            post: candidate.post.clone(),
            request: spec.request,
            source: spec.source.to_owned(),
            expected_playable_gain_ms: spec.playable_ms,
            utility: utility(candidate, spec.playable_ms, expected),
            authority: authority(candidate, self.snapshot, self.base.mode),
            commitment_until_ms: self
                .snapshot
                .observed_at_ms
                .saturating_add(self.snapshot.commitment_ms),
            reason: spec.reason,
        };
        self.normalize_playability(candidate, allocation)
    }

    pub(super) fn normalize_playability(
        &self,
        candidate: &CandidateSnapshot,
        mut allocation: Allocation,
    ) -> Allocation {
        if self.direct_playback_blocked(candidate) {
            allocation.expected_playable_gain_ms = 0;
            allocation.utility.additional_playable_ms = 0;
            allocation.utility.score = 0.0;
        }
        allocation
    }
}

fn utility(candidate: &CandidateSnapshot, playable_ms: u64, expected_ms: u64) -> CandidateUtility {
    let probability = candidate.view_probability.value();
    CandidateUtility {
        view_probability: probability,
        additional_playable_ms: playable_ms,
        expected_delivery_ms: expected_ms,
        score: probability * playable_ms as f64 / expected_ms.max(1) as f64,
    }
}

pub(super) fn source(candidate: &CandidateSnapshot) -> Option<&str> {
    candidate
        .origins
        .iter()
        .find(|item| item.available)
        .map(|item| item.source.as_str())
}

pub(super) fn classify(allocation: &Allocation) -> super::super::ActionKind {
    match allocation.request {
        RetrievalRequest::FetchWhole { contract, .. } => super::super::ActionKind::FetchWhole {
            maximum_bytes: contract.maximum_bytes(),
        },
        RetrievalRequest::FetchRange { bytes, .. } => classify_range(allocation.reason, bytes),
    }
}

fn classify_range(reason: AllocationReason, bytes: crate::ByteRange) -> super::super::ActionKind {
    match reason {
        AllocationReason::MediaBootstrap => super::super::ActionKind::Prefix(bytes),
        AllocationReason::MediaLayoutDiscovery if bytes.start > 0 => {
            super::super::ActionKind::Tail(bytes)
        }
        _ => super::super::ActionKind::FetchRange(bytes),
    }
}

fn kind_for_request(request: RetrievalRequest) -> super::super::ActionKind {
    match request {
        RetrievalRequest::FetchRange { bytes, .. } => super::super::ActionKind::FetchRange(bytes),
        RetrievalRequest::FetchWhole { contract, .. } => super::super::ActionKind::FetchWhole {
            maximum_bytes: contract.maximum_bytes(),
        },
    }
}

pub(super) fn resources(kind: &super::super::ActionKind) -> super::super::ResourceCost {
    let bytes = action_bytes(kind);
    let requests = u16::from(network_request(kind));
    super::super::ResourceCost::new(bytes, bytes, 0, requests)
}

pub(super) fn request_resources(request: RetrievalRequest) -> super::super::ResourceCost {
    let bytes = request.reserved_network_bytes();
    super::super::ResourceCost::new(bytes, bytes, 0, 1)
}

fn action_bytes(kind: &super::super::ActionKind) -> u64 {
    match kind {
        super::super::ActionKind::Prefix(range)
        | super::super::ActionKind::Tail(range)
        | super::super::ActionKind::FetchRange(range)
        | super::super::ActionKind::CacheUpgrade(range) => range.len(),
        super::super::ActionKind::FetchWhole { maximum_bytes } => *maximum_bytes,
        _ => 0,
    }
}

fn network_request(kind: &super::super::ActionKind) -> bool {
    matches!(
        kind,
        super::super::ActionKind::Head
            | super::super::ActionKind::Prefix(_)
            | super::super::ActionKind::Tail(_)
            | super::super::ActionKind::FetchRange(_)
            | super::super::ActionKind::FetchWhole { .. }
            | super::super::ActionKind::CacheUpgrade(_)
            | super::super::ActionKind::Hedge { .. }
    )
}

pub(super) fn authority(
    candidate: &CandidateSnapshot,
    snapshot: &PlayabilitySnapshot,
    mode: ControlMode,
) -> PreemptionAuthority {
    if candidate.post == snapshot.playback.current && mode == ControlMode::Emergency {
        return PreemptionAuthority::PlaybackCritical;
    }
    if candidate.feed_offset.magnitude() <= 1 {
        return PreemptionAuthority::Transition;
    }
    PreemptionAuthority::Speculative
}
