use super::{Catalog, HttpObservation, LearnedFacts};
use crate::representation::TransferIdentity;
use crate::PostId;

impl Catalog {
    /// Compatibility entry point for a post's primary source.
    pub fn learn(&mut self, post: &PostId, facts: LearnedFacts) -> bool {
        let identity = self
            .entries
            .get(post)
            .and_then(|entry| entry.meta.urls.first())
            .and_then(|source| self.transfer_identity(post, source));
        identity.is_some_and(|identity| {
            self.learn_action_response_observation_for(&identity, HttpObservation::legacy(facts))
        })
    }

    pub fn learn_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        self.learn_response_for(identity, facts)
    }

    pub fn learn_head_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        self.learn_head_observation_for(identity, HttpObservation::legacy(facts))
    }

    pub fn learn_response_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        self.learn_action_response_observation_for(identity, HttpObservation::legacy(facts))
    }
}
