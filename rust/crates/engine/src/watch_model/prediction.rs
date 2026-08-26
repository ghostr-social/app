use super::distribution::{DeadlineDistribution, WatchDistribution};

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateWatchPrediction {
    watch: WatchDistribution,
    play_start: DeadlineDistribution,
    reach_probability: f64,
}

impl CandidateWatchPrediction {
    pub(super) fn new(
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

    pub fn play_start(&self) -> &DeadlineDistribution {
        &self.play_start
    }

    pub fn reach_probability(&self) -> f64 {
        self.reach_probability
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WatchWindowPrediction {
    candidates: Vec<CandidateWatchPrediction>,
}

#[cfg(test)]
#[path = "prediction/test_support.rs"]
mod test_support;

impl WatchWindowPrediction {
    pub(super) fn new(candidates: Vec<CandidateWatchPrediction>) -> Self {
        Self { candidates }
    }

    pub fn candidates(&self) -> &[CandidateWatchPrediction] {
        &self.candidates
    }
}
