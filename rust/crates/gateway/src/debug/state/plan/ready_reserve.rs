use super::next_reserve::{hls_stage, infeasibility};
use super::{range, RangeSnapshot};
use ghostr_engine::adaptive::{
    ReadyReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct ReadyReserveSnapshot {
    target: usize,
    ready: usize,
    structural: usize,
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
    Structural {
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
    HlsReady {
        post_id: String,
    },
    HlsStructural {
        post_id: String,
    },
    HlsInFlight {
        post_id: String,
        stage: &'static str,
    },
    HlsPending {
        post_id: String,
        stage: &'static str,
    },
}

pub(super) fn snapshot(value: &ReadyReserveEvidence) -> ReadyReserveSnapshot {
    ReadyReserveSnapshot {
        target: value.target,
        ready: value.ready,
        structural: value.structural,
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
        ReserveCandidateState::Ready { .. } => ReserveCandidateSnapshot::Ready { post_id },
        ReserveCandidateState::Structural { .. } => {
            ReserveCandidateSnapshot::Structural { post_id }
        }
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
        ReserveCandidateState::HlsReady => ReserveCandidateSnapshot::HlsReady { post_id },
        ReserveCandidateState::HlsStructural => ReserveCandidateSnapshot::HlsStructural { post_id },
        ReserveCandidateState::HlsInFlight { stage } => ReserveCandidateSnapshot::HlsInFlight {
            post_id,
            stage: hls_stage(*stage),
        },
        ReserveCandidateState::HlsPending { stage } => ReserveCandidateSnapshot::HlsPending {
            post_id,
            stage: hls_stage(*stage),
        },
    }
}
