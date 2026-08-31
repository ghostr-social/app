use super::super::plan::{NextReserveInfeasibility, ReserveCandidateState};
use super::super::reserve_candidate::ReserveCandidate;
use super::super::reserve_model::{is_in_flight, is_ready, is_structural};
use super::super::{CandidateSnapshot, HlsBootstrapState, HlsCandidateSnapshot, PlayerPreparation};

pub(super) fn for_candidate(candidate: ReserveCandidate<'_>) -> ReserveCandidateState {
    match candidate {
        ReserveCandidate::Progressive(candidate) => progressive(candidate),
        ReserveCandidate::Hls(candidate) => hls(candidate),
    }
}

pub(super) fn is_ready_state(state: &ReserveCandidateState) -> bool {
    matches!(
        state,
        ReserveCandidateState::Ready { .. } | ReserveCandidateState::HlsReady
    )
}

fn progressive(candidate: &CandidateSnapshot) -> ReserveCandidateState {
    if is_ready(candidate) {
        ReserveCandidateState::Ready {
            startup: candidate.startup.clone().expect("ready startup"),
        }
    } else if is_structural(candidate) {
        ReserveCandidateState::Structural {
            startup: candidate.startup.clone().expect("structural startup"),
        }
    } else if is_in_flight(candidate) {
        ReserveCandidateState::InFlight
    } else {
        preparing(candidate)
    }
}

fn hls(candidate: &HlsCandidateSnapshot) -> ReserveCandidateState {
    if candidate.player_preparation == PlayerPreparation::Failed {
        return ReserveCandidateState::Infeasible {
            reason: NextReserveInfeasibility::PolicyDenied,
        };
    }
    match &candidate.state {
        HlsBootstrapState::Ready if candidate.player_ready() => ReserveCandidateState::HlsReady,
        HlsBootstrapState::Ready => ReserveCandidateState::HlsStructural,
        HlsBootstrapState::Active {
            stage,
            cancelling: false,
            ..
        } => ReserveCandidateState::HlsInFlight { stage: *stage },
        HlsBootstrapState::Active { stage, .. } | HlsBootstrapState::Pending { stage, .. } => {
            ReserveCandidateState::HlsPending { stage: *stage }
        }
        HlsBootstrapState::Failed => ReserveCandidateState::Infeasible {
            reason: NextReserveInfeasibility::PolicyDenied,
        },
    }
}

fn preparing(candidate: &CandidateSnapshot) -> ReserveCandidateState {
    let ranges = candidate
        .in_flight
        .iter()
        .filter(|active| active.identity_current && !active.cancelling)
        .map(|active| active.effective_bytes)
        .collect();
    if Vec::is_empty(&ranges) {
        ReserveCandidateState::Unprepared
    } else {
        ReserveCandidateState::Preparing { ranges }
    }
}
