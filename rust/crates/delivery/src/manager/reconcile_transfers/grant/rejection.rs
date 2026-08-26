use crate::delivery_events::DecisionToken;
use crate::manager::time;
use crate::manager::workers::PreparedTransfer;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::origin_model::ExplorationClaim;
use ghostr_engine::{ActionId, PostId};

impl DeliveryWorker {
    pub(super) async fn reject_commit(
        &mut self,
        prepared: PreparedTransfer,
        action: ActionId,
        bound: bool,
        exploration_claim: Option<ExplorationClaim>,
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
        self.release_exploration(exploration_claim);
        prepared.release(&self.ctx.store).await;
    }

    pub(super) async fn reject_binding(
        &mut self,
        prepared: PreparedTransfer,
        token: DecisionToken,
        exploration_claim: Option<ExplorationClaim>,
    ) {
        self.commands.resolve_decision_token(
            &token,
            DecisionOutcome::Failed {
                class: "decision_binding_rejected".into(),
                elapsed_ms: 0,
            },
        );
        self.release_exploration(exploration_claim);
        prepared.release(&self.ctx.store).await;
    }

    pub(super) fn reject_grant(
        &mut self,
        post: &PostId,
        error: &anyhow::Error,
        decision: Option<DecisionToken>,
        exploration_claim: Option<ExplorationClaim>,
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
        self.release_exploration(exploration_claim);
    }

    fn release_exploration(&mut self, claim: Option<ExplorationClaim>) {
        let Some(claim) = claim else { return };
        self.keeper
            .stats_mut()
            .origin_model_mut()
            .release_exploration(&claim);
    }
}
