use super::*;

impl DecisionRecord {
    pub(crate) fn emulate_legacy_warp_v2(&mut self) {
        self.schema_version = super::super::UNSEALED_WARP_SCHEMA_VERSION;
        self.replay_plan_hash = super::super::replay::warp_identity(self);
        self.terminal_evidence_hash = None;
    }

    pub(crate) fn emulate_legacy_policy_record(
        &mut self,
        allocation: &crate::adaptive::AllocationPlan,
        privacy: &crate::adaptive::DecisionPrivacy,
        schema_version: u16,
    ) {
        assert!(matches!(schema_version, 1 | 4));
        self.schema_version = schema_version;
        self.replay_plan_hash = super::super::plan_identity::capture_legacy(allocation, privacy);
    }
}
