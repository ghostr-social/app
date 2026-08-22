use super::{ChunkAttempt, OpenedResponse, ResponseObservation};
use crate::manager::DeliveryWorker;
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};

impl DeliveryWorker {
    pub(super) fn learn_opened_response(
        &mut self,
        attempt: &ChunkAttempt,
        response: &OpenedResponse,
        observed_at_ms: u64,
    ) {
        let (total, ranged) = response_evidence(response.observation());
        let headers = response.evidence();
        let facts = LearnedFacts {
            content_length: total,
            accept_ranges: ranged,
            host: ghostr_engine::host_stats::host_of(&headers.final_url),
        };
        let observation = HttpObservation::new(
            facts,
            headers.content_type.clone(),
            observed_at_ms,
            headers.validator.clone(),
        );
        self.state
            .catalog_mut()
            .learn_response_observation_for(attempt.identity(), observation);
    }
}

fn response_evidence(response: ResponseObservation) -> (Option<u64>, Option<bool>) {
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
