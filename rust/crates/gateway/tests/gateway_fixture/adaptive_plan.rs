use ghostr_engine::adaptive::{
    Allocation, AllocationPlan, AllocationReason, CandidateUtility, DiscoveryDemand,
    NextReserveEvidence, NextReserveInfeasibility, PreemptionAuthority, RetainedAllocation,
    RetrievalRequest,
};
use ghostr_engine::{ByteRange, PostId};

const SOURCE: &str = "https://media.example/p1.mp4";

pub fn plan() -> AllocationPlan {
    AllocationPlan {
        allocations: vec![allocation()],
        retained: vec![retained()],
        discovery_demand: DiscoveryDemand::Hold,
        next_reserve: NextReserveEvidence::Infeasible {
            post: PostId::new("p2"),
            reason: NextReserveInfeasibility::NoTransferBudget,
        },
        ..AllocationPlan::default()
    }
}

fn allocation() -> Allocation {
    Allocation {
        post: PostId::new("p1"),
        request: range_request(10, 20),
        source: SOURCE.to_owned(),
        expected_playable_gain_ms: 500,
        utility: utility(),
        authority: PreemptionAuthority::Transition,
        commitment_until_ms: 1_000,
        reason: AllocationReason::MediaBootstrap,
    }
}

fn retained() -> RetainedAllocation {
    RetainedAllocation {
        action_id: ghostr_engine::ActionId::new(7),
        post: PostId::new("p0"),
        request: range_request(0, 10),
        source: SOURCE.to_owned(),
        utility: utility(),
        committed_until_ms: 1_000,
        reason: AllocationReason::UsefulCommitment,
    }
}

fn range_request(start: u64, end: u64) -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(start, end),
        promotion: None,
    }
}

fn utility() -> CandidateUtility {
    CandidateUtility {
        view_probability: 0.4,
        additional_playable_ms: 500,
        expected_delivery_ms: 20,
        score: 10.0,
    }
}
