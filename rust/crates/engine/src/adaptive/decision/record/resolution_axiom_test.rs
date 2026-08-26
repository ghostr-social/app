use super::*;

impl DecisionRecord {
    pub(crate) fn emulate_legacy_warp_v2(&mut self) {
        self.schema_version = super::super::UNSEALED_WARP_SCHEMA_VERSION;
        self.replay_plan_hash = super::super::replay::warp_identity(self);
        self.terminal_evidence_hash = None;
    }
}
