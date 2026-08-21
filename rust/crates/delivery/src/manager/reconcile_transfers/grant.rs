use crate::delivery_events::DecisionToken;
use crate::manager::plan::PlannedTransfer;
use crate::manager::workers::PreparedTransfer;
use crate::manager::{origin_admission, time, DeliveryWorker};
use ghostr_engine::adaptive::DecisionOutcome;

#[cfg(test)]
#[path = "grant/origin_concurrency_test.rs"]
mod origin_concurrency_test;

struct AdmittedGrant {
    transfer: PlannedTransfer,
    observed_at_ms: u64,
}

impl DeliveryWorker {
    pub(super) async fn grant(
        &mut self,
        transfer: PlannedTransfer,
        decision: &mut Option<DecisionToken>,
    ) -> Option<ghostr_engine::ActionId> {
        if !self.grant_eligible(&transfer) {
            return None;
        }
        let admitted = self.admit_origin(transfer)?;
        self.prepare_grant(admitted, decision).await
    }

    fn grant_eligible(&self, transfer: &PlannedTransfer) -> bool {
        let post = &transfer.request.chunk.post;
        !self.downloads.contains_transfer(transfer)
            && !self.retry.is_cooling(post)
            && !self.pressure.is_parked()
    }

    async fn prepare_grant(
        &mut self,
        admitted: AdmittedGrant,
        decision: &mut Option<DecisionToken>,
    ) -> Option<ghostr_engine::ActionId> {
        let post = admitted.transfer.request.chunk.post.clone();
        match self.downloads.prepare(&self.ctx, admitted.transfer).await {
            Ok(prepared) => {
                self.bind_and_launch(prepared, admitted.observed_at_ms, decision)
                    .await
            }
            Err(error) => {
                self.reject_grant(&post, &error, decision.take());
                None
            }
        }
    }

    async fn bind_and_launch(
        &mut self,
        prepared: PreparedTransfer,
        observed_at_ms: u64,
        decision: &mut Option<DecisionToken>,
    ) -> Option<ghostr_engine::ActionId> {
        let action = prepared.action();
        if let Some(token) = decision.take() {
            if !self.commands.bind_decision(&token, action, observed_at_ms) {
                self.reject_binding(prepared, token).await;
                return None;
            }
        }
        Some(self.downloads.launch(self.ctx.clone(), prepared))
    }

    async fn reject_binding(&mut self, prepared: PreparedTransfer, token: DecisionToken) {
        self.commands.resolve_decision_token(
            &token,
            DecisionOutcome::Failed {
                class: "decision_binding_rejected".into(),
                elapsed_ms: 0,
            },
        );
        prepared.release(&self.ctx.store).await;
    }

    fn admit_origin(&mut self, transfer: PlannedTransfer) -> Option<AdmittedGrant> {
        let observed_at_ms = time::unix_time_ms();
        let authority = ghostr_engine::RequestAuthority::from_url(&transfer.url)?;
        let concurrency = origin_concurrency(&self.ctx.requests, &authority);
        let query = origin_admission::query(&transfer, observed_at_ms, concurrency);
        let mode = origin_admission::mode(&transfer);
        let admission =
            self.keeper
                .stats_mut()
                .origin_model_mut()
                .claim(&query, observed_at_ms, mode);
        origin_admission::apply(transfer, admission).map(|transfer| AdmittedGrant {
            transfer,
            observed_at_ms,
        })
    }

    fn reject_grant(
        &mut self,
        post: &ghostr_engine::PostId,
        error: &anyhow::Error,
        decision: Option<DecisionToken>,
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
    }
}

fn origin_concurrency(
    requests: &ghostr_net::media_request_executor::MediaRequestExecutor,
    authority: &ghostr_engine::RequestAuthority,
) -> usize {
    requests
        .active_for(authority)
        .saturating_add(1)
        .min(requests.limits().per_authority())
}
