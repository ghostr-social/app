use crate::chunk::downloader::{ResponseObservation, ResponseRejection};
use crate::manager::inflight::ChunkAttempt;
use crate::manager::transfers::ObservedResponse;
use crate::manager::DeliveryWorker;
use ghostr_net::media_log_identity::MediaLogIdentity;

impl DeliveryWorker {
    pub(super) async fn observe_response(&mut self, observed: ObservedResponse) {
        let response = observed.response.observation();
        if !self.downloads.observe_headers(
            &observed.attempt,
            &observed.response,
            crate::manager::time::unix_time_ms(),
        ) {
            return;
        }
        let outcome = match response {
            ResponseObservation::Ignored { .. } => {
                if observed.response.evidence().validator.is_none() {
                    self.record_independent_object(&observed.attempt);
                }
                self.adopt_response_generation(&observed.attempt, &observed.response)
                    .await
                    .map(|admission| admission.is_some())
            }
            ResponseObservation::Rejected(ResponseRejection::Semantics) => {
                self.reject_response_generation(&observed.attempt, &observed.response)
                    .await
            }
            ResponseObservation::Rejected(_)
            | ResponseObservation::Partial { .. }
            | ResponseObservation::Body { .. } => return,
        };
        self.finish_observed_generation(&observed.attempt, outcome);
    }

    fn finish_observed_generation(
        &mut self,
        attempt: &ChunkAttempt,
        outcome: anyhow::Result<bool>,
    ) {
        if outcome.as_ref().is_ok_and(|applied| *applied) {
            return;
        }
        self.downloads.reject_response(attempt);
        if let Err(error) = outcome {
            log::warn!(
                "Could not apply HTTP generation for {}: {error:#}",
                MediaLogIdentity::from_url(attempt.identity().source().as_str())
            );
        }
    }
}
