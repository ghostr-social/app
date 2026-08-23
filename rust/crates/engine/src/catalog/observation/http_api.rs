use super::{Catalog, HttpAuthority};
use crate::catalog::HttpObservation;
use crate::representation::{
    HttpGenerationAuthority, HttpGenerationLease, HttpGenerationStamp, TransferIdentity,
};

impl Catalog {
    pub fn learn_head_observation_for(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> bool {
        self.learn_head_observation_with_stamp_for(identity, observation)
            .is_some()
    }

    pub fn learn_head_observation_with_stamp_for(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> Option<HttpGenerationStamp> {
        self.learn_http_identity(identity, observation, HttpAuthority::Head)
            .then(|| self.http_generation_stamp_for(identity))
            .flatten()
    }

    pub fn learn_response_observation_for(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> bool {
        self.learn_http_identity(identity, observation, HttpAuthority::Response)
    }

    pub fn learn_action_response_observation_for(
        &mut self,
        identity: &TransferIdentity,
        observation: HttpObservation,
    ) -> bool {
        self.learn_action_http_identity(identity, observation)
    }

    pub fn http_generation_for(&self, identity: &TransferIdentity) -> Option<HttpGenerationLease> {
        match self.http_generation_stamp_for(identity)?.authority() {
            HttpGenerationAuthority::Trusted(lease) => Some(lease.clone()),
            HttpGenerationAuthority::Unknown(_) => None,
        }
    }

    pub fn http_generation_stamp_for(
        &self,
        identity: &TransferIdentity,
    ) -> Option<HttpGenerationStamp> {
        let entry = self.entries.get(identity.post())?;
        (entry.binding.transfer(identity.source().as_str()).as_ref() == Some(identity))
            .then(|| entry.http_generation_stamp(identity.source().as_str()))
            .flatten()
    }

    pub fn reject_response_generation_for(
        &mut self,
        identity: &TransferIdentity,
        final_url: &str,
        validator: Option<crate::evidence::EvidenceValidator>,
        observed: crate::evidence::EvidenceTime,
    ) -> Option<HttpGenerationAuthority> {
        let entry = self.entries.get_mut(identity.post())?;
        if entry.binding.transfer(identity.source().as_str()).as_ref() != Some(identity) {
            return None;
        }
        let authority = entry.reject_response_generation(
            identity.source().as_str(),
            final_url,
            validator,
            observed,
        )?;
        entry.evidence_clock_ms = entry.evidence_clock_ms.max(observed.observed_at_ms);
        Some(authority)
    }
}
