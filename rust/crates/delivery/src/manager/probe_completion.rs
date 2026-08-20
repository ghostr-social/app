//! Completion policy for media-size probes.

use crate::manager::failure::{classify, FailureClass};
use crate::manager::transfers::ProbeDone;
use crate::manager::DeliveryWorker;
use crate::probe::media::ProbeResult;
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::host_stats::host_of;
use ghostr_engine::representation::TransferIdentity;
use ghostr_net::media_log_identity::MediaLogIdentity;
use log::warn;

impl DeliveryWorker {
    pub(crate) async fn finish_probe(&mut self, done: ProbeDone) {
        let identity = self
            .probes
            .current_identity(self.state.catalog(), &done.post, &done.url);
        let Some(identity) = identity else {
            self.probes.release(&done.post);
            return;
        };
        self.keeper.note_probe(&done);
        match done.outcome {
            Ok(result) => self.finish_probe_result(&identity, result).await,
            Err(error) => self.finish_probe_error(&identity, error),
        }
    }

    async fn finish_probe_result(&mut self, identity: &TransferIdentity, result: ProbeResult) {
        if result.content_length.is_some_and(|length| length > 0) {
            self.probes.learned(identity.post());
            self.absorb_probe(identity, result);
            return;
        }
        if self.defer_probe_to_body(identity) {
            return;
        }
        self.probes.release(identity.post());
        let source = identity.source().as_str();
        warn!(
            "Probe did not reveal a usable content length for {}",
            MediaLogIdentity::from_url(source)
        );
        self.note_failed_attempt(identity.post(), source, FailureClass::Transient);
    }

    fn finish_probe_error(&mut self, identity: &TransferIdentity, error: anyhow::Error) {
        let class = classify(&error);
        warn!(
            "Probe failed for {} ({class:?})",
            MediaLogIdentity::from_url(identity.source().as_str())
        );
        if self.defer_probe_to_body(identity) {
            return;
        }
        self.probes.release(identity.post());
        self.note_failed_attempt(identity.post(), identity.source().as_str(), class);
    }

    fn defer_probe_to_body(&mut self, identity: &TransferIdentity) -> bool {
        if !self.downloads.contains_identity(identity) {
            return false;
        }
        self.probes.defer_to_body(identity.post());
        true
    }

    fn absorb_probe(&mut self, identity: &TransferIdentity, result: ProbeResult) {
        let source = identity.source().as_str();
        self.note_successful_attempt(identity.post(), source);
        let facts = LearnedFacts {
            content_length: result.content_length,
            accept_ranges: result.accept_ranges,
            host: host_of(source),
        };
        let observation = HttpObservation::new(
            facts,
            result.content_type,
            crate::manager::time::unix_time_ms(),
            result.validator,
        );
        self.state
            .catalog_mut()
            .learn_head_observation_for(identity, observation);
    }
}
