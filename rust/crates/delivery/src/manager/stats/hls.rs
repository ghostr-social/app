use super::StatsKeeper;
use ghostr_engine::host_stats::host_of;
use ghostr_engine::origin_model::{OriginObservation, OriginOutcome};

impl StatsKeeper {
    pub(crate) fn note_hls(&mut self, observation: OriginObservation) {
        if let Some(host) = host_of(observation.query.url()) {
            match observation.outcome {
                OriginOutcome::Success => self.stats.record_success(&host),
                OriginOutcome::Failure(_) => self.stats.record_failure(&host),
                OriginOutcome::Cancelled => return,
            }
        }
        self.stats.origin_model_mut().observe(observation);
        self.dirty = true;
    }
}
