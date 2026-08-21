use super::{
    DecisionResolution, DecisionToken, LegacyDecisionPublication, WarpDecisionPublication,
};
use crate::delivery_events::CommandReceiver;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::ActionId;

impl CommandReceiver {
    pub(crate) fn publish_decision(
        &self,
        publication: LegacyDecisionPublication<'_>,
    ) -> Option<DecisionToken> {
        self.decisions.publish(publication.into())
    }

    pub(crate) fn publish_warp_decision(
        &self,
        publication: WarpDecisionPublication<'_>,
    ) -> Option<DecisionToken> {
        self.decisions.publish(publication.into())
    }

    pub(crate) fn bind_decision(
        &self,
        token: &DecisionToken,
        action: ActionId,
        observed_at_ms: u64,
    ) -> bool {
        self.decisions.bind(token, action, observed_at_ms)
    }

    pub(crate) fn resolve_decision(
        &self,
        action: ActionId,
        outcome: DecisionOutcome,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        self.decisions.resolve(action, outcome, observed_at_ms)
    }

    pub(crate) fn resolve_decision_token(
        &self,
        token: &DecisionToken,
        outcome: DecisionOutcome,
    ) -> bool {
        self.decisions.resolve_token(token, outcome)
    }
}
