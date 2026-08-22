use super::progressive_startup;
use ghostr_engine::adaptive::{
    AllocationPlan, ControlMode, NextReserveInfeasibility, ReadyReserveEvidence,
    ReserveCandidateEvidence, ReserveCandidateState,
};
use ghostr_engine::{ByteRange, PostId};

pub fn plan() -> AllocationPlan {
    AllocationPlan {
        mode: ControlMode::Safety,
        ready_reserve: ReadyReserveEvidence {
            target: 3,
            ready: 1,
            structural: 1,
            protected: 3,
            recovery_horizon_ms: 1_800,
            underflow_risk_bps: 420,
            ready_coverage_ms: 2_300,
            candidates: vec![ready(), structural(), planned(), unavailable()],
        },
        ..AllocationPlan::default()
    }
}

fn ready() -> ReserveCandidateEvidence {
    evidence(
        "p1",
        ReserveCandidateState::Ready {
            startup: progressive_startup(),
        },
    )
}

fn structural() -> ReserveCandidateEvidence {
    evidence(
        "p2",
        ReserveCandidateState::Structural {
            startup: progressive_startup(),
        },
    )
}

fn planned() -> ReserveCandidateEvidence {
    evidence(
        "p3",
        ReserveCandidateState::Planned {
            ranges: vec![ByteRange::new(0, 8)],
        },
    )
}

fn unavailable() -> ReserveCandidateEvidence {
    evidence(
        "p4",
        ReserveCandidateState::Infeasible {
            reason: NextReserveInfeasibility::NoLiveOrigin,
        },
    )
}

fn evidence(post: &str, state: ReserveCandidateState) -> ReserveCandidateEvidence {
    ReserveCandidateEvidence {
        post: PostId::new(post),
        state,
    }
}
