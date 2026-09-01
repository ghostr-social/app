use ghostr_engine::adaptive::{
    Allocation, AllocationPlan, AllocationReason, CandidateUtility, PreemptionAuthority,
    RetainedAllocation, RetrievalRequest,
};
use ghostr_engine::{ActionId, ByteRange, PostId};

pub(super) fn plan(target_eta_ms: u64) -> AllocationPlan {
    AllocationPlan {
        allocations: vec![
            allocation("clip", 0, 5),
            allocation("clip", 2_000, target_eta_ms),
            allocation("other", 2_000, 10),
        ],
        retained: vec![retained("clip", 2_000, 225)],
        ..AllocationPlan::default()
    }
}

fn allocation(post: &str, gain_ms: u64, eta_ms: u64) -> Allocation {
    Allocation {
        post: PostId::new(post),
        request: request(),
        source: "source".into(),
        expected_playable_gain_ms: gain_ms,
        utility: utility(gain_ms, eta_ms),
        authority: PreemptionAuthority::Transition,
        commitment_until_ms: 1_000,
        reason: AllocationReason::CurrentBufferReserve,
    }
}

fn retained(post: &str, gain_ms: u64, eta_ms: u64) -> RetainedAllocation {
    RetainedAllocation {
        action_id: ActionId::new(1),
        post: PostId::new(post),
        request: request(),
        source: "source".into(),
        utility: utility(gain_ms, eta_ms),
        committed_until_ms: 1_000,
        reason: AllocationReason::CurrentBufferReserve,
    }
}

fn utility(gain_ms: u64, eta_ms: u64) -> CandidateUtility {
    CandidateUtility {
        view_probability: 1.0,
        additional_playable_ms: gain_ms,
        expected_delivery_ms: eta_ms,
        score: 1.0,
    }
}

fn request() -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 16),
        promotion: None,
    }
}
