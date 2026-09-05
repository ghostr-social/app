use super::{ChunkAttempt, OpenedResponse, ResponseObservation};
use crate::manager::DeliveryWorker;
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::representation::{HttpGenerationAuthority, HttpGenerationLease};

impl DeliveryWorker {
    pub(crate) fn learn_opened_response(
        &mut self,
        attempt: &ChunkAttempt,
        response: &OpenedResponse,
        observed: ghostr_engine::evidence::EvidenceTime,
    ) -> Option<HttpGenerationLease> {
        let observation = opened_observation(response, observed)?;
        self.state
            .catalog_mut()
            .learn_response_observation_for(attempt.identity(), observation)
            .then(|| self.state.catalog().http_generation_for(attempt.identity()))
            .flatten()
    }

    pub(crate) fn learn_action_scoped_response(
        &mut self,
        attempt: &ChunkAttempt,
        response: &OpenedResponse,
        observed: ghostr_engine::evidence::EvidenceTime,
    ) -> bool {
        let Some(observation) = opened_observation(response, observed) else {
            return false;
        };
        self.state
            .catalog_mut()
            .learn_action_response_observation_for(attempt.identity(), observation)
    }

    pub(crate) fn reject_opened_generation(
        &mut self,
        attempt: &ChunkAttempt,
        response: &OpenedResponse,
    ) -> Option<HttpGenerationAuthority> {
        let headers = response.evidence();
        self.state.catalog_mut().reject_response_generation_for(
            attempt.identity(),
            &headers.final_url,
            headers.validator.clone(),
            headers.observed,
        )
    }
}

fn opened_observation(
    response: &OpenedResponse,
    observed: ghostr_engine::evidence::EvidenceTime,
) -> Option<HttpObservation> {
    let (total, ranged) = response_evidence(response.observation())?;
    let headers = response.evidence();
    Some(
        HttpObservation::new(
            LearnedFacts {
                content_length: total,
                accept_ranges: ranged,
                host: ghostr_engine::host_stats::host_of(&headers.final_url),
            },
            headers.content_type.clone(),
            observed,
            headers.validator.clone(),
        )
        .with_final_url(headers.final_url.clone())
        .with_request_selection(headers.request_selection),
    )
}

fn response_evidence(response: ResponseObservation) -> Option<(Option<u64>, Option<bool>)> {
    match response {
        ResponseObservation::Rejected(_) => None,
        ResponseObservation::Partial { total, .. } => Some((total, Some(true))),
        ResponseObservation::Body {
            total,
            range_support,
            ..
        }
        | ResponseObservation::Ignored {
            total,
            range_support,
        } => Some((total, range_support)),
    }
}
