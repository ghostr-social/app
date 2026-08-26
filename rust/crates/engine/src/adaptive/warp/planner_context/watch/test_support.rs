use super::*;

impl PlannerWatchEvidence {
    pub const fn reach_probability_bps(self) -> Option<u16> {
        match self {
            Self::Unavailable => None,
            Self::Learned {
                reach_probability_bps,
                ..
            } => Some(reach_probability_bps),
        }
    }
}
