use super::{
    DecisionClaim, DecisionResolution, DecisionToken, LegacyDecisionPublication,
    RequestDecisionBinding, WarpDecisionPublication,
};
use crate::delivery_events::CommandReceiver;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::ActionId;

impl CommandReceiver {
    pub(crate) fn claim_decision(
        &self,
        token: DecisionToken,
        identity: &TransferIdentity,
        started_at_ms: u64,
    ) -> Result<DecisionClaim, DecisionToken> {
        self.decisions.claim(token, identity, started_at_ms)
    }

    pub(crate) fn resolve_decision_claim(
        &self,
        claim: DecisionClaim,
        outcome: DecisionOutcome,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        self.decisions.resolve_claim(claim, outcome, observed_at_ms)
    }

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

    pub(crate) fn bind_request_decision(
        &self,
        token: &DecisionToken,
        binding: RequestDecisionBinding<'_>,
    ) -> bool {
        self.decisions.bind_request(token, binding)
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
