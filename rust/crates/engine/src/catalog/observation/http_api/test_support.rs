use super::*;

impl Catalog {
    pub fn learn_head_observation_for(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> bool {
        self.learn_head_observation_with_stamp_for(identity, observation)
            .is_some()
    }
}
