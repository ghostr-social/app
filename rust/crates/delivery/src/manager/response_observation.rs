use crate::chunk::downloader::ResponseObservation;
use crate::manager::transfers::ObservedResponse;
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(super) fn observe_response(&mut self, observed: ObservedResponse) {
        if !self
            .downloads
            .observe_response(&observed.attempt, observed.response)
        {
            return;
        }
        let identity = observed.attempt.identity();
        let (total, ranged) = evidence(observed.response);
        self.learn_response_evidence(identity, total, ranged);
    }
}

fn evidence(response: ResponseObservation) -> (Option<u64>, Option<bool>) {
    match response {
        ResponseObservation::Partial { total, .. } => (total, Some(true)),
        ResponseObservation::Body {
            total,
            range_support,
            ..
        }
        | ResponseObservation::Ignored {
            total,
            range_support,
        } => (total, range_support),
    }
}
