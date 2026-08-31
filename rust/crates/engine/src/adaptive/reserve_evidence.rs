use super::plan::{
    NextReserveEvidence, NextReserveInfeasibility, ReserveCandidateEvidence, ReserveCandidateState,
};
use super::reserve_candidate::ReserveCandidate;

mod state;

pub(super) fn initial(candidates: &[ReserveCandidate<'_>]) -> Vec<ReserveCandidateEvidence> {
    candidates
        .iter()
        .map(|candidate| ReserveCandidateEvidence {
            post: candidate.post().clone(),
            kind: candidate.kind(),
            state: state::for_candidate(*candidate),
        })
        .collect()
}

pub(super) fn count_ready(evidence: &[ReserveCandidateEvidence]) -> usize {
    evidence
        .iter()
        .filter(|item| state::is_ready_state(&item.state))
        .count()
}

pub(super) fn count_ordered_ready(evidence: &[ReserveCandidateEvidence]) -> usize {
    evidence
        .iter()
        .take_while(|item| state::is_ready_state(&item.state))
        .count()
}

pub(super) fn count_structural(evidence: &[ReserveCandidateEvidence]) -> usize {
    evidence
        .iter()
        .filter(|item| {
            matches!(
                item.state,
                ReserveCandidateState::Structural { .. }
                    | ReserveCandidateState::HlsStructural
            )
        })
        .count()
}

pub(super) fn count_protected(evidence: &[ReserveCandidateEvidence]) -> usize {
    evidence
        .iter()
        .filter(|item| is_protected_state(&item.state))
        .count()
}

pub(super) fn is_protected_state(state: &ReserveCandidateState) -> bool {
    matches!(
        state,
        ReserveCandidateState::Ready { .. }
            | ReserveCandidateState::Structural { .. }
            | ReserveCandidateState::InFlight
            | ReserveCandidateState::Planned { .. }
            | ReserveCandidateState::HlsReady
            | ReserveCandidateState::HlsStructural
            | ReserveCandidateState::HlsInFlight { .. }
    )
}

pub(super) fn reject_first_unprepared(evidence: &mut [ReserveCandidateEvidence]) {
    if let Some(item) = evidence
        .iter_mut()
        .find(|item| {
            matches!(
                item.state,
                ReserveCandidateState::Unprepared | ReserveCandidateState::HlsPending { .. }
            )
        })
    {
        item.state = ReserveCandidateState::Infeasible {
            reason: NextReserveInfeasibility::CurrentUnprotected,
        };
    }
}

pub(super) fn immediate_next(evidence: &[ReserveCandidateEvidence]) -> NextReserveEvidence {
    let Some(item) = evidence.first() else {
        return NextReserveEvidence::NotApplicable;
    };
    match &item.state {
        ReserveCandidateState::Ready { startup } => ready(item, startup),
        ReserveCandidateState::Structural { startup } => structural(item, startup),
        ReserveCandidateState::InFlight => in_flight(item),
        ReserveCandidateState::Preparing { ranges } | ReserveCandidateState::Planned { ranges } => {
            granted(item, ranges)
        }
        ReserveCandidateState::Infeasible { reason } => infeasible(item, *reason),
        ReserveCandidateState::HlsReady => NextReserveEvidence::HlsReady {
            post: item.post.clone(),
        },
        ReserveCandidateState::HlsStructural => NextReserveEvidence::HlsStructural {
            post: item.post.clone(),
        },
        ReserveCandidateState::HlsInFlight { stage } => NextReserveEvidence::HlsInFlight {
            post: item.post.clone(),
            stage: *stage,
        },
        ReserveCandidateState::HlsPending { stage } => NextReserveEvidence::HlsPending {
            post: item.post.clone(),
            stage: *stage,
        },
        ReserveCandidateState::Unprepared | ReserveCandidateState::Probing => {
            NextReserveEvidence::NotApplicable
        }
    }
}

fn structural(
    item: &ReserveCandidateEvidence,
    startup: &crate::media_timeline::StartupFootprint,
) -> NextReserveEvidence {
    NextReserveEvidence::Structural {
        post: item.post.clone(),
        startup: startup.clone(),
    }
}

fn ready(
    item: &ReserveCandidateEvidence,
    startup: &crate::media_timeline::StartupFootprint,
) -> NextReserveEvidence {
    NextReserveEvidence::Ready {
        post: item.post.clone(),
        startup: startup.clone(),
    }
}

fn in_flight(item: &ReserveCandidateEvidence) -> NextReserveEvidence {
    NextReserveEvidence::InFlight {
        post: item.post.clone(),
    }
}

fn granted(item: &ReserveCandidateEvidence, ranges: &[crate::ByteRange]) -> NextReserveEvidence {
    ranges
        .first()
        .map_or(NextReserveEvidence::NotApplicable, |range| {
            NextReserveEvidence::Granted {
                post: item.post.clone(),
                range: *range,
            }
        })
}

fn infeasible(
    item: &ReserveCandidateEvidence,
    reason: NextReserveInfeasibility,
) -> NextReserveEvidence {
    NextReserveEvidence::Infeasible {
        post: item.post.clone(),
        reason,
    }
}
