use super::{Catalog, CatalogEntry, CompleteBytesObservation, HttpObservation};
use crate::representation::TransferIdentity;
use crate::PostId;

mod http;
mod http_api;
pub(in crate::catalog) use http::HttpGenerationRecord;
mod mirror;
pub(super) use mirror::VerifiedMirrorRecord;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum HttpAuthority {
    Head,
    Response,
    CompleteBytes,
}

pub(super) struct HttpLearning {
    observed_at_ms: u64,
    labels: Vec<crate::evidence::CalibrationLabel>,
}

impl Catalog {
    pub fn learn_complete_bytes_for(
        &mut self,
        identity: &TransferIdentity,
        complete: CompleteBytesObservation,
    ) -> bool {
        let final_url = complete.final_url.clone();
        let generation = complete.generation.clone();
        let mut observation = HttpObservation::new(
            super::LearnedFacts {
                content_length: Some(complete.total_bytes.get()),
                host: crate::host_stats::host_of(&complete.final_url),
                ..super::LearnedFacts::default()
            },
            None,
            complete.observed,
            complete.validator,
        )
        .with_final_url(final_url);
        if let Some(generation) = generation {
            observation = observation.with_generation(generation);
        }
        self.learn_http_identity(identity, observation, HttpAuthority::CompleteBytes)
    }

    pub fn record_verified_hash_for(
        &mut self,
        identity: &TransferIdentity,
        digest: &str,
        origin: &str,
        observed: impl Into<crate::evidence::EvidenceTime>,
    ) -> bool {
        if !self.identity_claims_digest(identity, digest) {
            return false;
        }
        self.record_hash_match(identity, digest, origin, observed.into());
        true
    }

    pub fn record_verified_hash_for_generation(
        &mut self,
        identity: &TransferIdentity,
        digest: &str,
        origin: &str,
        observed: impl Into<crate::evidence::EvidenceTime>,
        generation: &crate::representation::HttpGenerationLease,
    ) -> bool {
        if !self.identity_claims_digest(identity, digest)
            || self.http_generation_for(identity).as_ref() != Some(generation)
            || !generation
                .key()
                .validator()
                .is_some_and(|value| value.is_strong())
        {
            return false;
        }
        self.record_hash_match(identity, digest, origin, observed.into());
        let stamp = crate::representation::HttpGenerationStamp::from_trusted(generation.clone());
        self.entries.get_mut(identity.post()).is_some_and(|entry| {
            entry.record_verified_mirror(identity.source().as_str(), digest, stamp)
        })
    }

    /// Rejects only this presentation's failed endpoint, never a digest's other claimants.
    pub fn quarantine_source(
        &mut self,
        identity: &TransferIdentity,
        digest: &str,
        observed_at_ms: u64,
    ) -> Vec<PostId> {
        if !self.identity_claims_digest(identity, digest) {
            return Vec::new();
        }
        self.reject_source(identity, digest);
        if let Some(entry) = self.entries.get_mut(identity.post()) {
            entry.record_source_mismatch(digest, identity.source().as_str(), observed_at_ms);
        }
        self.apply_known_quarantine(identity.post());
        vec![identity.post().clone()]
    }

    fn learn_http_identity(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
        authority: HttpAuthority,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(identity.post()) else {
            return false;
        };
        if entry.binding.transfer(identity.source().as_str()).as_ref() != Some(identity) {
            return false;
        }
        let Some(learning) = entry.learn_http(identity.source().as_str(), observation, authority)
        else {
            return false;
        };
        self.observe_labels(learning.labels, learning.observed_at_ms);
        true
    }

    fn learn_action_http_identity(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(identity.post()) else {
            return false;
        };
        if entry.binding.transfer(identity.source().as_str()).as_ref() != Some(identity) {
            return false;
        }
        let Some(learning) = entry.learn_action_http(identity.source().as_str(), observation)
        else {
            return false;
        };
        self.observe_labels(learning.labels, learning.observed_at_ms);
        true
    }

    fn identity_claims_digest(&self, identity: &TransferIdentity, digest: &str) -> bool {
        self.entries.get(identity.post()).is_some_and(|entry| {
            entry.binding.transfer(identity.source().as_str()).as_ref() == Some(identity)
                && advertised_digest(entry, digest)
        })
    }

    fn record_hash_match(
        &mut self,
        identity: &TransferIdentity,
        digest: &str,
        origin: &str,
        observed: crate::evidence::EvidenceTime,
    ) {
        let Some(entry) = self.entries.get_mut(identity.post()) else {
            return;
        };
        let labels = entry.hash_labels(digest, true, observed.observed_at_ms);
        entry.record_integrity(digest, origin, observed);
        self.observe_labels(labels, observed.observed_at_ms);
    }
}

fn advertised_digest(entry: &CatalogEntry, digest: &str) -> bool {
    entry
        .meta
        .sha256
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case(digest))
}
