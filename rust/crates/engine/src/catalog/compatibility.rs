use super::{Catalog, HttpObservation, LearnedFacts};
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
}

#[cfg(any(test, feature = "test"))]
#[path = "compatibility/test_support.rs"]
mod test_support;
