use super::{is_admission_timeout, StatsKeeper};
use crate::manager::transfers::ChunkDone;
use ghostr_engine::host_stats::host_of;

impl StatsKeeper {
    /// Mirrors the downloader's recording rules on the owned stats.
    pub fn note_chunk(&mut self, done: &ChunkDone) {
        if !done.request_started || is_admission_timeout(&done.outcome) {
            return;
        }
        let local = done
            .outcome
            .as_ref()
            .err()
            .is_some_and(crate::chunk::sink::is_local_store_failure);
        let policy_stop = done
            .outcome
            .as_ref()
            .err()
            .is_some_and(crate::chunk::whole_body_policy::is);
        if local && done.whole_body_completion.is_none() {
            return;
        }
        let Some(host) = host_of(&done.url) else {
            return;
        };
        match &done.outcome {
            Ok(_) => self.stats.record_success(&host),
            Err(_) if local || policy_stop => self.stats.record_success(&host),
            Err(_) => self.stats.record_failure(&host),
        }
        if let Some(observation) = &done.origin {
            self.stats.origin_model_mut().observe(observation);
        }
        if let Some(observation) = &done.open_body {
            self.stats.origin_model_mut().observe_open_body(observation);
        }
        self.dirty = true;
    }
}
