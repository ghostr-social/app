use super::super::plan::{NextReserveInfeasibility, ReserveCandidateState};
use super::super::{CandidateSnapshot, MediaLayout};

pub(super) struct ScheduleOutcome {
    pub(super) state: ReserveCandidateState,
    pub(super) protected: bool,
}

pub(super) fn probing(candidate: &CandidateSnapshot) -> ScheduleOutcome {
    let state = match candidate.layout {
        MediaLayout::Unknown => ReserveCandidateState::Probing,
        _ => ReserveCandidateState::Unprepared,
    };
    ScheduleOutcome {
        state,
        protected: false,
    }
}

pub(super) fn unavailable(reason: NextReserveInfeasibility) -> ScheduleOutcome {
    ScheduleOutcome {
        state: ReserveCandidateState::Infeasible { reason },
        protected: false,
    }
}
