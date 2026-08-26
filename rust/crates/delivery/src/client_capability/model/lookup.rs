use super::ClientCapabilityModel;
use crate::client_capability::inference::{inferred_support, status_for};
use crate::client_capability::{ClientCapabilityProfile, ClientCapabilityStatus};

impl ClientCapabilityModel {
    pub(crate) fn status(
        &self,
        generation: u64,
        profile: &ClientCapabilityProfile,
    ) -> ClientCapabilityStatus {
        if self.generation != Some(generation) {
            return ClientCapabilityStatus::Unknown;
        }
        let exact = self.content_status(generation, profile);
        if exact != ClientCapabilityStatus::Unknown {
            return exact;
        }
        inferred_support(&self.records, profile).unwrap_or(ClientCapabilityStatus::Unknown)
    }

    pub(crate) fn content_status(
        &self,
        generation: u64,
        profile: &ClientCapabilityProfile,
    ) -> ClientCapabilityStatus {
        if self.generation != Some(generation) {
            return ClientCapabilityStatus::Unknown;
        }
        if let Some(record) = self
            .records
            .iter()
            .rev()
            .find(|record| record.profile.applies_to(profile))
        {
            return status_for(&record.result);
        }
        if self
            .active
            .iter()
            .any(|active| active.profile.applies_to(profile))
        {
            return ClientCapabilityStatus::Testing;
        }
        ClientCapabilityStatus::Unknown
    }
}

#[cfg(test)]
#[path = "lookup_axiom_test.rs"]
pub(crate) mod axiom_test_support;
