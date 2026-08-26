use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UnitDeadline {
    deadline: DeadlineDistribution,
    reach_probability: f64,
}

impl UnitDeadline {
    pub(crate) fn deadline(&self) -> &DeadlineDistribution {
        &self.deadline
    }

    pub(crate) fn reach_probability(&self) -> f64 {
        self.reach_probability
    }
}

impl CandidateWatchPrediction {
    pub(crate) fn unit_deadline(&self, offset_ms: u64) -> UnitDeadline {
        UnitDeadline {
            deadline: self.play_start.shifted(offset_ms),
            reach_probability: self.reach_probability * self.watch.survival(offset_ms),
        }
    }
}
