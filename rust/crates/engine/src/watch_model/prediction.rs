use super::distribution::{DeadlineDistribution, WatchDistribution};

#[derive(Clone, Debug, PartialEq)]
pub struct UnitDeadline {
    deadline: DeadlineDistribution,
    reach_probability: f64,
}

impl UnitDeadline {
    pub fn deadline(&self) -> &DeadlineDistribution {
        &self.deadline
    }

    pub fn reach_probability(&self) -> f64 {
        self.reach_probability
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateWatchPrediction {
    watch: WatchDistribution,
    play_start: DeadlineDistribution,
    reach_probability: f64,
}

impl CandidateWatchPrediction {
    pub(crate) fn new(
        watch: WatchDistribution,
        play_start: DeadlineDistribution,
        reach_probability: f64,
    ) -> Self {
        Self {
            watch,
            play_start,
            reach_probability: reach_probability.clamp(0.0, 1.0),
        }
    }

    pub fn watch(&self) -> &WatchDistribution {
        &self.watch
    }

    pub fn play_start(&self) -> &DeadlineDistribution {
        &self.play_start
    }

    pub fn reach_probability(&self) -> f64 {
        self.reach_probability
    }

    pub fn unit_deadline(&self, offset_ms: u64) -> UnitDeadline {
        UnitDeadline {
            deadline: self.play_start.shifted(offset_ms),
            reach_probability: self.reach_probability * self.watch.survival(offset_ms),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WatchWindowPrediction {
    candidates: Vec<CandidateWatchPrediction>,
    model_revision: u64,
}

impl WatchWindowPrediction {
    pub(crate) fn new(candidates: Vec<CandidateWatchPrediction>, model_revision: u64) -> Self {
        Self {
            candidates,
            model_revision,
        }
    }

    pub fn candidates(&self) -> &[CandidateWatchPrediction] {
        &self.candidates
    }

    pub fn model_revision(&self) -> u64 {
        self.model_revision
    }
}
