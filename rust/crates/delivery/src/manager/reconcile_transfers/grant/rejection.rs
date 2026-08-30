use crate::delivery_events::DecisionToken;
use crate::manager::time;
use crate::manager::workers::PreparedTransfer;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::origin_model::{AdmissionClaim, AdmissionClaimTerminal};
use ghostr_engine::{ActionId, PostId};

use super::disposition::{outcome_for_rejection, GrantRejection};

impl DeliveryWorker {
    pub(super) fn reject_selection(
        &self,
        decision: &mut Option<DecisionToken>,
        rejection: GrantRejection,
    ) {
        if let Some(token) = decision.take() {
            self.commands
                .resolve_decision_token(&token, outcome_for_rejection(rejection));
        }
    }

    pub(super) async fn reject_commit(
        &mut self,
        prepared: PreparedTransfer,
        action: ActionId,
        bound: bool,
        admission_claim: Option<AdmissionClaim>,
    ) {
        if bound {
            self.commands.resolve_decision(
                action,
                DecisionOutcome::Failed {
                    class: "warp_resource_commit_rejected".into(),
                    elapsed_ms: 0,
                },
                time::unix_time_ms(),
            );
        }
        self.release_admission(admission_claim);
        prepared.release(&self.ctx.store).await;
    }

    pub(super) async fn reject_binding(
        &mut self,
        prepared: PreparedTransfer,
        token: DecisionToken,
        admission_claim: Option<AdmissionClaim>,
    ) {
        self.commands.resolve_decision_token(
            &token,
            DecisionOutcome::Failed {
                class: "decision_binding_rejected".into(),
                elapsed_ms: 0,
            },
        );
        self.release_admission(admission_claim);
        prepared.release(&self.ctx.store).await;
    }

    pub(super) async fn reject_superseded(
        &mut self,
        prepared: PreparedTransfer,
        action: ActionId,
        bound: bool,
        admission_claim: Option<AdmissionClaim>,
    ) {
        if bound {
            self.commands.resolve_decision(
                action,
                DecisionOutcome::Superseded,
                time::unix_time_ms(),
            );
        }
        self.release_admission(admission_claim);
        prepared.release(&self.ctx.store).await;
    }

    pub(super) fn reject_grant(
        &mut self,
        post: &PostId,
        error: &anyhow::Error,
        decision: Option<DecisionToken>,
        admission_claim: Option<AdmissionClaim>,
    ) {
        if let Some(token) = decision {
            self.commands.resolve_decision_token(
                &token,
                DecisionOutcome::Failed {
                    class: format!("{:?}", crate::manager::failure::classify(error)),
                    elapsed_ms: 0,
                },
            );
        }
        if !self.absorb_store_pressure(post, error) {
            log::warn!("Could not reserve a video action: {error:#}");
        }
        self.release_admission(admission_claim);
    }

    fn release_admission(&mut self, claim: Option<AdmissionClaim>) {
        let Some(claim) = claim else { return };
        self.keeper
            .stats_mut()
            .origin_model_mut()
            .complete_claim(claim, AdmissionClaimTerminal::NotStarted);
        self.keeper.mark_origin_model_changed();
    }
}
