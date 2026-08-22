use super::{Catalog, CatalogEntry, HttpObservation};
use crate::representation::TransferIdentity;
use crate::PostId;

mod http;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum HttpAuthority {
    Head,
    Response,
    CompleteBytes,
}

pub(super) struct HttpLearning {
    pub(super) observed_at_ms: u64,
    pub(super) labels: Vec<crate::evidence::CalibrationLabel>,
}

impl Catalog {
    pub fn learn_head_observation_for(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> bool {
        self.learn_http_identity(identity, observation, HttpAuthority::Head)
    }

    pub fn learn_response_observation_for(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> bool {
        self.learn_http_identity(identity, observation, HttpAuthority::Response)
    }

    pub fn learn_complete_bytes_for(
        &mut self,
        identity: &TransferIdentity,
        total_bytes: u64,
        observed_at_ms: u64,
    ) -> bool {
        let observation = HttpObservation::new(
            super::LearnedFacts {
                content_length: Some(total_bytes),
                ..super::LearnedFacts::default()
            },
            None,
            observed_at_ms,
            None,
        );
        if !self.learn_http_identity(identity, observation, HttpAuthority::CompleteBytes) {
            return false;
        }
        if let Some(digest) = self
            .entries
            .get(identity.post())
            .and_then(|entry| entry.meta.sha256.clone())
        {
            self.record_hash_match(identity, &digest, observed_at_ms);
        }
        true
    }

    pub fn quarantine_mirror_group(
        &mut self,
        identity: &TransferIdentity,
        digest: &str,
        observed_at_ms: u64,
    ) -> Vec<PostId> {
        if !self.identity_claims_digest(identity, digest) {
            return Vec::new();
        }
        if self.quarantined_digests.insert(digest.to_ascii_lowercase()) {
            self.reliability_revision = self.reliability_revision.saturating_add(1);
        }
        let origin = identity.source().as_str().to_owned();
        let labels: Vec<_> = self
            .entries
            .values()
            .filter(|entry| advertised_digest(entry, digest))
            .flat_map(|entry| entry.hash_labels(digest, false, observed_at_ms))
            .collect();
        let mut posts: Vec<_> = self
            .digest_claims
            .get(&digest.to_ascii_lowercase())
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        for post in &posts {
            if let Some(entry) = self.entries.get_mut(post) {
                entry.quarantine_integrity(digest, &origin, observed_at_ms);
            }
        }
        posts.sort();
        self.observe_labels(labels, observed_at_ms);
        posts
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
        let learning = entry.learn_http(identity.source().as_str(), observation, authority);
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
        observed_at_ms: u64,
    ) {
        let Some(entry) = self.entries.get_mut(identity.post()) else {
            return;
        };
        let labels = entry.hash_labels(digest, true, observed_at_ms);
        entry.record_integrity(digest, identity.source().as_str(), observed_at_ms);
        self.observe_labels(labels, observed_at_ms);
    }
}

fn advertised_digest(entry: &CatalogEntry, digest: &str) -> bool {
    entry
        .meta
        .sha256
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case(digest))
}
