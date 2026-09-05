use crate::adaptive::{
    Allocation, AllocationReason, CandidateUtility, GeneratedAction, PlannerCommand,
    PreemptionAuthority, PromotionGrant, RetrievalRequest,
};
use crate::ByteRange;
pub(super) const SOURCE: &str = "https://origin.example/media";
pub(super) const RESERVED_BYTES: u64 = 800_000;
pub(super) fn assert_immediate(action: &GeneratedAction) {
    let request = match &action.command {
        PlannerCommand::Transfer(work) | PlannerCommand::Hedge { transfer: work, .. } => {
            work.request
        }
        _ => panic!("expected request command"),
    };
    let immediate = request.immediate_network_bytes();
    assert_ne!(request.reserved_network_bytes(), immediate);
    assert_eq!(action.node.resources.network_bytes, immediate);
    assert_eq!(action.node.resources.storage_bytes, immediate);
    assert_eq!(action.node.resources.cpu_ms, 0);
    assert_eq!(action.node.resources.requests, 1);
}

pub(super) fn request() -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 64_000),
        promotion: Some(PromotionGrant {
            maximum_bytes: RESERVED_BYTES,
            valid_until_ms: 20_000,
        }),
    }
}

pub(super) fn allocation(post: crate::PostId) -> Allocation {
    Allocation {
        post,
        request: request(),
        source: SOURCE.to_owned(),
        expected_playable_gain_ms: 1_000,
        utility: CandidateUtility {
            view_probability: 1.0,
            additional_playable_ms: 1_000,
            expected_delivery_ms: 10,
            score: 1.0,
        },
        authority: PreemptionAuthority::PlaybackCritical,
        commitment_until_ms: 20_000,
        reason: AllocationReason::MediaBootstrap,
    }
}
