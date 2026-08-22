use crate::adaptive::{
    Allocation, AllocationReason, CandidateUtility, PreemptionAuthority, RetrievalRequest,
};
use crate::PostId;

pub(crate) fn allocation(source: &str, request: RetrievalRequest) -> Allocation {
    Allocation {
        post: PostId::new("secret-post"),
        request,
        source: source.into(),
        expected_playable_gain_ms: 1_000,
        utility: CandidateUtility {
            view_probability: 1.0,
            additional_playable_ms: 1_000,
            expected_delivery_ms: 10,
            score: 1.0,
        },
        authority: PreemptionAuthority::PlaybackCritical,
        commitment_until_ms: 1_000,
        reason: AllocationReason::MediaBootstrap,
    }
}
