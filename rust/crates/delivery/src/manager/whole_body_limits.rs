use ghostr_engine::adaptive::WholeBodyExhaustion;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::{HttpGenerationStamp, TransferIdentity};
use std::collections::HashMap;

#[cfg(test)]
#[path = "whole_body_limits/generation_test.rs"]
mod generation_test;

#[derive(Clone)]
struct Exhaustion {
    evidence: WholeBodyExhaustion,
    generation: Option<HttpGenerationStamp>,
}

#[derive(Default)]
pub(super) struct WholeBodyLimits {
    exhausted: HashMap<TransferIdentity, Exhaustion>,
}

impl WholeBodyLimits {
    pub(super) fn record(
        &mut self,
        catalog: &Catalog,
        identity: TransferIdentity,
        maximum_bytes: u64,
        observed_bytes: u64,
        generation: Option<HttpGenerationStamp>,
    ) -> bool {
        if !generation_is_current(catalog, &identity, &generation) {
            return false;
        }
        let Some(evidence) = merged_evidence(
            self.exhausted.get(&identity),
            maximum_bytes,
            observed_bytes,
            generation.as_ref(),
        ) else {
            return false;
        };
        self.exhausted.insert(
            identity,
            Exhaustion {
                evidence,
                generation,
            },
        );
        true
    }

    pub(super) fn current(
        &mut self,
        catalog: &Catalog,
    ) -> HashMap<TransferIdentity, WholeBodyExhaustion> {
        self.exhausted.retain(|identity, entry| {
            catalog
                .transfer_identity(identity.post(), identity.source().as_str())
                .as_ref()
                == Some(identity)
                && catalog.http_generation_stamp_for(identity) == entry.generation
        });
        self.exhausted
            .iter()
            .map(|(identity, entry)| (identity.clone(), entry.evidence))
            .collect()
    }

    pub(super) fn clear(&mut self) {
        self.exhausted.clear();
    }
}

fn merged_evidence(
    prior: Option<&Exhaustion>,
    maximum_bytes: u64,
    observed_bytes: u64,
    generation: Option<&HttpGenerationStamp>,
) -> Option<WholeBodyExhaustion> {
    let matching = prior.filter(|entry| entry.generation.as_ref() == generation);
    let maximum = matching.map_or(maximum_bytes, |entry| {
        entry.evidence.maximum_bytes().max(maximum_bytes)
    });
    let observed = matching.map_or(observed_bytes, |entry| {
        entry.evidence.observed_bytes().max(observed_bytes)
    });
    WholeBodyExhaustion::new(maximum, observed)
}

fn generation_is_current(
    catalog: &Catalog,
    identity: &TransferIdentity,
    generation: &Option<HttpGenerationStamp>,
) -> bool {
    catalog
        .transfer_identity(identity.post(), identity.source().as_str())
        .as_ref()
        == Some(identity)
        && catalog.http_generation_stamp_for(identity).as_ref() == generation.as_ref()
}
