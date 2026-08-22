use super::DecisionRecord;
use crate::adaptive::{DecisionOutcome, RecordedResourceCost, ResourceCost};

impl DecisionRecord {
    pub fn resolve(&mut self, outcome: DecisionOutcome) -> bool {
        self.resolve_terminal(outcome, None)
    }

    pub fn resolve_with_resources(
        &mut self,
        outcome: DecisionOutcome,
        resources: ResourceCost,
    ) -> bool {
        self.resolve_terminal(outcome, Some(resources.into()))
    }

    fn resolve_terminal(
        &mut self,
        outcome: DecisionOutcome,
        resources: Option<RecordedResourceCost>,
    ) -> bool {
        if outcome == DecisionOutcome::Pending || self.eventual_outcome != DecisionOutcome::Pending
        {
            return false;
        }
        self.eventual_outcome = outcome;
        self.actual_resources = resources;
        self.terminal_evidence_hash = Some(super::replay::terminal_identity(self));
        true
    }

    #[cfg(test)]
    pub(crate) fn emulate_legacy_warp_v2(&mut self) {
        self.schema_version = super::UNSEALED_WARP_SCHEMA_VERSION;
        self.replay_plan_hash = super::replay::warp_identity(self);
        self.terminal_evidence_hash = None;
    }
}
