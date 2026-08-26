use crate::chunk::downloader::HttpResponseEvidence;
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use core::num::NonZeroU64;
use ghostr_engine::catalog::CompleteBytesObservation;
use ghostr_engine::representation::{HttpGenerationLease, TransferIdentity};
use ghostr_partial_store::partial_range_completion::Completion;

pub(super) struct FinalizedEvidence<'a> {
    pub total: Option<u64>,
    pub response: Option<&'a HttpResponseEvidence>,
    pub advertised: Option<&'a str>,
    pub completion: Completion,
    pub generation: Option<&'a HttpGenerationLease>,
}

impl DeliveryWorker {
    pub(super) fn learn_network_completion(
        &mut self,
        done: &ChunkDone,
        generation: Option<&HttpGenerationLease>,
    ) -> bool {
        let Some(completion) = done.whole_body_completion.as_ref() else {
            return false;
        };
        let observation = exact_generation(completion.observation().clone(), generation);
        self.state
            .catalog_mut()
            .learn_complete_bytes_for(done.attempt.identity(), observation)
    }

    pub(super) fn learn_finalized(
        &mut self,
        identity: &TransferIdentity,
        finalized: &FinalizedEvidence<'_>,
    ) {
        let observed = crate::manager::time::evidence_time();
        let exact_generation_accepted = complete_observation(
            finalized.total,
            finalized.response,
            finalized.generation,
            observed,
        )
        .is_some_and(|exact| {
            self.state
                .catalog_mut()
                .learn_complete_bytes_for(identity, exact)
        });
        if finalized.completion == Completion::Verified {
            let generation = exact_generation_accepted
                .then_some(finalized.generation)
                .flatten();
            self.learn_verified_hash(
                identity,
                finalized.response,
                finalized.advertised,
                generation,
                observed,
            );
        }
    }

    fn learn_verified_hash(
        &mut self,
        identity: &TransferIdentity,
        response: Option<&HttpResponseEvidence>,
        advertised: Option<&str>,
        generation: Option<&HttpGenerationLease>,
        observed: ghostr_engine::evidence::EvidenceTime,
    ) {
        let Some(digest) = advertised else { return };
        let origin = response.map_or_else(|| identity.source().as_str(), |item| &item.final_url);
        let catalog = self.state.catalog_mut();
        if generation.is_some_and(|value| {
            catalog.record_verified_hash_for_generation(identity, digest, origin, observed, value)
        }) {
            return;
        }
        catalog.record_verified_hash_for(identity, digest, origin, observed);
    }
}

fn complete_observation(
    total: Option<u64>,
    response: Option<&HttpResponseEvidence>,
    generation: Option<&HttpGenerationLease>,
    observed: ghostr_engine::evidence::EvidenceTime,
) -> Option<CompleteBytesObservation> {
    let response = response?;
    let observation = CompleteBytesObservation::new(
        NonZeroU64::new(total?)?,
        response.final_url.clone(),
        observed,
        response.validator.clone(),
    );
    Some(exact_generation(observation, generation))
}

fn exact_generation(
    observation: CompleteBytesObservation,
    generation: Option<&HttpGenerationLease>,
) -> CompleteBytesObservation {
    match generation {
        Some(lease) => observation.with_generation(lease.clone()),
        None => observation,
    }
}
