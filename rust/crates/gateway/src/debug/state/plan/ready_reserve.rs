use super::next_reserve::infeasibility;
use super::{range, RangeSnapshot};
use ghostr_engine::adaptive::{
    ReadyReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct ReadyReserveSnapshot {
    target: usize,
    ready: usize,
    protected: usize,
    recovery_horizon_ms: u64,
    underflow_risk_bps: u16,
    ready_coverage_ms: u64,
    candidates: Vec<ReserveCandidateSnapshot>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ReserveCandidateSnapshot {
    Unprepared {
        post_id: String,
    },
    Ready {
        post_id: String,
    },
    InFlight {
        post_id: String,
    },
    Probing {
        post_id: String,
    },
    Preparing {
        post_id: String,
        ranges: Vec<RangeSnapshot>,
    },
    Planned {
        post_id: String,
        ranges: Vec<RangeSnapshot>,
    },
    Infeasible {
        post_id: String,
        reason: &'static str,
    },
}

pub(super) fn snapshot(value: &ReadyReserveEvidence) -> ReadyReserveSnapshot {
    ReadyReserveSnapshot {
        target: value.target,
        ready: value.ready,
        protected: value.protected,
        recovery_horizon_ms: value.recovery_horizon_ms,
        underflow_risk_bps: value.underflow_risk_bps,
        ready_coverage_ms: value.ready_coverage_ms,
        candidates: value.candidates.iter().map(candidate).collect(),
    }
}

fn candidate(value: &ReserveCandidateEvidence) -> ReserveCandidateSnapshot {
    let post_id = value.post.as_str().to_owned();
    match &value.state {
        ReserveCandidateState::Unprepared => ReserveCandidateSnapshot::Unprepared { post_id },
        ReserveCandidateState::Ready => ReserveCandidateSnapshot::Ready { post_id },
        ReserveCandidateState::InFlight => ReserveCandidateSnapshot::InFlight { post_id },
        ReserveCandidateState::Probing => ReserveCandidateSnapshot::Probing { post_id },
        ReserveCandidateState::Preparing { ranges } => ReserveCandidateSnapshot::Preparing {
            post_id,
            ranges: ranges.iter().copied().map(range).collect(),
        },
        ReserveCandidateState::Planned { ranges } => ReserveCandidateSnapshot::Planned {
            post_id,
            ranges: ranges.iter().copied().map(range).collect(),
        },
        ReserveCandidateState::Infeasible { reason } => ReserveCandidateSnapshot::Infeasible {
            post_id,
            reason: infeasibility(*reason),
        },
    }
}
