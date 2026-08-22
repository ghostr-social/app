use super::{observation, Catalog, HttpObservation, LearnedFacts};
use crate::representation::TransferIdentity;
use crate::PostId;

impl Catalog {
    /// Compatibility entry point for a post's primary source.
    pub fn learn(&mut self, post: &PostId, facts: LearnedFacts) -> bool {
        let Some(entry) = self.entries.get_mut(post) else {
            return false;
        };
        let Some(source) = entry.meta.urls.first().cloned() else {
            return false;
        };
        let learning = entry.learn_http(
            &source,
            HttpObservation::legacy(facts),
            observation::HttpAuthority::Response,
        );
        self.observe_labels(learning.labels, learning.observed_at_ms);
        true
    }

    pub fn learn_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        self.learn_response_for(identity, facts)
    }

    pub fn learn_head_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        self.learn_head_observation_for(identity, HttpObservation::legacy(facts))
    }

    pub fn learn_response_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        self.learn_response_observation_for(identity, HttpObservation::legacy(facts))
    }
}
