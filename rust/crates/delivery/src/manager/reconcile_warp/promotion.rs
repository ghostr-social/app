use super::WarpDirective;
use crate::delivery_events::DecisionToken;
use crate::manager::inflight::{PromotionRejection, PromotionTarget};
use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use crate::manager::{time, DeliveryWorker};
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};
use ghostr_partial_store::partial_range_store::ActionReservationExtension;

#[cfg(test)]
#[path = "promotion/outcome_test.rs"]
mod outcome_test;

struct PreparedPromotion {
    preflight: crate::manager::inflight::PromotionPreflight,
    extension: ActionReservationExtension,
}

impl DeliveryWorker {
    pub(super) async fn promote_selected(
        &mut self,
        directive: &WarpDirective,
        decision: Option<DecisionToken>,
        commit: &mut Option<SelectedCommit>,
    ) {
        let started_at_ms = time::unix_time_ms();
        let result = self.try_promote_selected(directive, commit).await;
        let elapsed_ms = time::unix_time_ms().saturating_sub(started_at_ms);
        if let Some(token) = decision {
            self.commands
                .resolve_decision_token(&token, outcome(result, elapsed_ms));
        }
    }

    async fn try_promote_selected(
        &mut self,
        directive: &WarpDirective,
        commit: &mut Option<SelectedCommit>,
    ) -> Result<(), &'static str> {
        let target = self
            .promotion_target(directive)
            .ok_or("warp_promotion_identity_missing")?;
        let prepared = self.prepare_promotion(&target).await?;
        self.commit_promotion(prepared, commit).await
    }

    async fn prepare_promotion(
        &mut self,
        target: &PromotionTarget,
    ) -> Result<PreparedPromotion, &'static str> {
        let preflight = self
            .downloads
            .preflight_promotion(target, time::unix_time_ms())
            .map_err(PromotionRejection::class)?;
        let store = std::sync::Arc::clone(&self.ctx.store);
        let extension = store
            .extend_action(preflight.store_action(), target.maximum_bytes())
            .await
            .map_err(|error| {
                log::warn!("WARP promotion store extension was rejected: {error:#}");
                "warp_promotion_store_extension_rejected"
            })?;
        if extension.additional_bytes() != preflight.additional_bytes()
            || !self
                .downloads
                .activate_promotion(&preflight, time::unix_time_ms())
        {
            rollback_store(&store, extension).await;
            return Err("warp_promotion_activation_rejected");
        }
        Ok(PreparedPromotion {
            preflight,
            extension,
        })
    }

    async fn commit_promotion(
        &mut self,
        prepared: PreparedPromotion,
        commit: &mut Option<SelectedCommit>,
    ) -> Result<(), &'static str> {
        let delta = prepared.preflight.additional_bytes();
        let resources = ResourceCost::new(delta, delta, 0, 0);
        match self.commit_selected(commit, resources, time::unix_time_ms()) {
            CommitResult::Committed => self.finish_promotion_commit(prepared, delta).await,
            CommitResult::Rejected | CommitResult::Untracked => {
                self.downloads.rollback_promotion(&prepared.preflight);
                rollback_store(&self.ctx.store, prepared.extension).await;
                Err("warp_resource_commit_rejected")
            }
        }
    }

    async fn finish_promotion_commit(
        &mut self,
        prepared: PreparedPromotion,
        delta: u64,
    ) -> Result<(), &'static str> {
        if self.downloads.commit_promotion_network(&prepared.preflight) {
            prepared.extension.commit();
            self.request_immediate_replan();
            return Ok(());
        }
        self.warp_planner
            .reconcile_network_reservation(delta, 0, time::unix_time_ms());
        self.downloads.rollback_promotion(&prepared.preflight);
        rollback_store(&self.ctx.store, prepared.extension).await;
        Err("warp_promotion_reservation_lost")
    }

    fn promotion_target(&self, directive: &WarpDirective) -> Option<PromotionTarget> {
        let WarpDirective::Promote {
            post,
            action,
            source,
            grant,
        } = directive
        else {
            return None;
        };
        let identity = self.state.catalog().transfer_identity(post, source)?;
        Some(PromotionTarget::new(*action, identity, *grant))
    }
}

impl PromotionRejection {
    fn class(self) -> &'static str {
        match self {
            Self::Expired => "warp_promotion_expired",
            Self::Missing => "warp_promotion_action_missing",
            Self::StaleIdentity => "warp_promotion_identity_stale",
            Self::ResponseOpened => "warp_promotion_response_opened",
            Self::ResponseNotPromotable => "warp_promotion_response_not_promotable",
            Self::AlreadyActivated => "warp_promotion_already_activated",
            Self::GrantMismatch => "warp_promotion_grant_mismatch",
            Self::InvalidDelta => "warp_promotion_invalid_delta",
            Self::Unavailable => "warp_promotion_action_unavailable",
        }
    }
}

async fn rollback_store(
    store: &ghostr_partial_store::partial_range_store::PartialRangeStore,
    extension: ghostr_partial_store::partial_range_store::ActionReservationExtension,
) {
    if let Err(error) = store.rollback_action(extension).await {
        log::error!("Could not roll back WARP promotion reservation: {error:#}");
    }
}

fn outcome(result: Result<(), &'static str>, elapsed_ms: u64) -> DecisionOutcome {
    match result {
        Ok(()) => DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms,
        },
        Err(class) => DecisionOutcome::Failed {
            class: class.to_owned(),
            elapsed_ms,
        },
    }
}
