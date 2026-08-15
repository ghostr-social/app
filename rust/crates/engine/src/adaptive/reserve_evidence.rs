use super::plan::{
    NextReserveEvidence, NextReserveInfeasibility, ReserveCandidateEvidence, ReserveCandidateState,
};
use super::reserve_model::{is_in_flight, is_ready};
use super::CandidateSnapshot;

pub(super) fn initial(candidates: &[&CandidateSnapshot]) -> Vec<ReserveCandidateEvidence> {
    candidates
        .iter()
        .map(|candidate| ReserveCandidateEvidence {
            post: candidate.post.clone(),
            state: initial_state(candidate),
        })
        .collect()
}

pub(super) fn count_ready(evidence: &[ReserveCandidateEvidence]) -> usize {
    evidence
        .iter()
        .filter(|item| item.state == ReserveCandidateState::Ready)
        .count()
}

pub(super) fn count_protected(evidence: &[ReserveCandidateEvidence]) -> usize {
    evidence
        .iter()
        .filter(|item| {
            matches!(
                item.state,
                ReserveCandidateState::Ready
                    | ReserveCandidateState::InFlight
                    | ReserveCandidateState::Planned { .. }
            )
        })
        .count()
}

pub(super) fn reject_first_unprepared(evidence: &mut [ReserveCandidateEvidence]) {
    if let Some(item) = evidence
        .iter_mut()
        .find(|item| item.state == ReserveCandidateState::Unprepared)
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
        ReserveCandidateState::Ready => ready(item),
        ReserveCandidateState::InFlight => in_flight(item),
        ReserveCandidateState::Preparing { ranges } => granted(item, ranges),
        ReserveCandidateState::Planned { ranges } => granted(item, ranges),
        ReserveCandidateState::Infeasible { reason } => infeasible(item, *reason),
        ReserveCandidateState::Unprepared | ReserveCandidateState::Probing => {
            NextReserveEvidence::NotApplicable
        }
    }
}

fn initial_state(candidate: &CandidateSnapshot) -> ReserveCandidateState {
    if is_ready(candidate) {
        ReserveCandidateState::Ready
    } else if is_in_flight(candidate) {
        ReserveCandidateState::InFlight
    } else {
        preparing_state(candidate)
    }
}

fn preparing_state(candidate: &CandidateSnapshot) -> ReserveCandidateState {
    let ranges: Vec<_> = candidate
        .in_flight
        .iter()
        .filter(|active| active.identity_current)
        .map(|active| active.bytes)
        .collect();
    match ranges.is_empty() {
        true => ReserveCandidateState::Unprepared,
        false => ReserveCandidateState::Preparing { ranges },
    }
}

fn ready(item: &ReserveCandidateEvidence) -> NextReserveEvidence {
    NextReserveEvidence::Ready {
        post: item.post.clone(),
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
