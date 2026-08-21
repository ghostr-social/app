use super::DecisionRecord;
use crate::adaptive::decision::advanced::{
    capture_executed, executed_coherent, RecordedWarpCommand,
};
use crate::adaptive::{DecisionOutcome, DecisionPrivacy, ExecutedRequest};
use crate::ActionId;

impl DecisionRecord {
    pub fn bind_action(&mut self, action: ActionId) -> bool {
        if !self.binding_available() || self.selected_is_request() {
            return false;
        }
        self.chosen_action_id = Some(action.value());
        true
    }

    pub fn bind_executed_action(
        &mut self,
        action: ActionId,
        request: &ExecutedRequest,
        privacy: &DecisionPrivacy,
    ) -> bool {
        if !self.binding_available() || self.executed_request.is_some() {
            return false;
        }
        let Some(selected) = self.selected() else {
            return false;
        };
        let executed = capture_executed(request, privacy);
        if !request.has_exact_resources() || !executed_coherent(&executed, selected) {
            return false;
        }
        self.chosen_action_id = Some(action.value());
        self.executed_request = Some(executed);
        true
    }

    fn binding_available(&self) -> bool {
        self.chosen_action.is_some()
            && self.chosen_action_id.is_none()
            && self.eventual_outcome == DecisionOutcome::Pending
    }

    fn selected(&self) -> Option<&crate::adaptive::RecordedWarpAction> {
        self.warp_decision.as_ref()?.selected.as_ref()
    }

    fn selected_is_request(&self) -> bool {
        self.selected().is_some_and(|selected| {
            matches!(
                selected.command,
                RecordedWarpCommand::Transfer { .. } | RecordedWarpCommand::Hedge { .. }
            )
        })
    }
}
