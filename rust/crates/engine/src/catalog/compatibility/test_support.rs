use super::*;
use crate::representation::TransferIdentity;

impl Catalog {
    #[cfg(test)]
    pub(crate) fn learn_head_for(
        &mut self,
        identity: &TransferIdentity,
        facts: LearnedFacts,
    ) -> bool {
        self.learn_head_observation_for(identity, HttpObservation::legacy(facts))
    }

    pub fn learn_response_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        self.learn_action_response_observation_for(identity, HttpObservation::legacy(facts))
    }
}
