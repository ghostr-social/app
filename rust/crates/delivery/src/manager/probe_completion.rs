//! Completion policy for media-size probes.

use crate::manager::failure::{classify, FailureClass};
use crate::manager::transfers::ProbeDone;
use crate::manager::DeliveryWorker;
use crate::probe::media::ProbeResult;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
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
            Err(error) => self.finish_probe_error(&done.post, &done.url, error),
        }
    }

    async fn finish_probe_result(&mut self, identity: &TransferIdentity, result: ProbeResult) {
        if result.content_length.is_some_and(|length| length > 0) {
            self.probes.learned(identity.post());
            self.absorb_probe(identity, result).await;
            return;
        }
        self.probes.release(identity.post());
        let source = identity.source().as_str();
        warn!("Probe did not reveal a usable content length for {source}");
        self.note_failed_attempt(identity.post(), source, FailureClass::Transient);
    }

    fn finish_probe_error(&mut self, post: &PostId, url: &str, error: anyhow::Error) {
        self.probes.release(post);
        warn!("Probe failed: {error:#}");
        self.note_failed_attempt(post, url, classify(&error));
    }

    async fn absorb_probe(&mut self, identity: &TransferIdentity, result: ProbeResult) {
        let source = identity.source().as_str();
        self.note_successful_attempt(identity.post(), source);
        self.learn_identity(identity, result.content_length, result.accept_ranges)
            .await;
    }
}
