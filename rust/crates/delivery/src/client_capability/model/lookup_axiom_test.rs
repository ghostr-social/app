use super::*;

impl ClientCapabilityModel {
    pub(crate) fn bounded_test_allowed(
        &self,
        generation: u64,
        profile: &ClientCapabilityProfile,
    ) -> bool {
        self.status(generation, profile) == ClientCapabilityStatus::Unknown
    }
}
