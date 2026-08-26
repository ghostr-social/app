use crate::delivery_events::{DecisionToken, RequestDecisionBinding};
use crate::manager::plan::PlannedTransfer;
use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use crate::manager::workers::PreparedTransfer;
use crate::manager::{origin_admission, time, DeliveryWorker};
use ghostr_engine::adaptive::{ExecutedRequest, ResourceCost, RetrievalRequest};
use ghostr_engine::origin_model::ExplorationClaim;

mod admitted;
use admitted::AdmittedGrant;
mod rejection;

#[cfg(test)]
#[path = "grant/immediate_resources_test.rs"]
mod immediate_resources_test;
#[cfg(test)]
#[path = "grant/origin_concurrency_test.rs"]
mod origin_concurrency_test;

struct PreparedGrant {
    transfer: PreparedTransfer,
    executed: ExecutedRequest,
    resources: ResourceCost,
    observed_at_ms: u64,
    exploration_claim: Option<ExplorationClaim>,
}

impl DeliveryWorker {
    pub(super) async fn grant(
        &mut self,
        transfer: PlannedTransfer,
        decision: &mut Option<DecisionToken>,
        selected: &mut Option<SelectedCommit>,
    ) -> Option<ghostr_engine::ActionId> {
        if !self.grant_eligible(&transfer) {
            return None;
        }
        let admitted = self.admit_origin(transfer)?;
        self.prepare_grant(admitted, decision, selected).await
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
        selected: &mut Option<SelectedCommit>,
    ) -> Option<ghostr_engine::ActionId> {
        let post = admitted.transfer.request.chunk.post.clone();
        let exploration_claim = admitted.exploration_claim.clone();
        match self.downloads.prepare(&self.ctx, admitted.transfer).await {
            Ok(transfer) => {
                self.bind_and_launch(
                    PreparedGrant {
                        transfer,
                        executed: admitted.executed,
                        resources: admitted.resources,
                        observed_at_ms: admitted.observed_at_ms,
                        exploration_claim: admitted.exploration_claim,
                    },
                    decision,
                    selected,
                )
                .await
            }
            Err(error) => {
                self.reject_grant(&post, &error, decision.take(), exploration_claim);
                None
            }
        }
    }

    async fn bind_and_launch(
        &mut self,
        mut prepared: PreparedGrant,
        decision: &mut Option<DecisionToken>,
        selected: &mut Option<SelectedCommit>,
    ) -> Option<ghostr_engine::ActionId> {
        let action = prepared.transfer.action();
        let bound = decision.is_some();
        if let Some(token) = decision.take() {
            let binding =
                RequestDecisionBinding::new(action, &prepared.executed, prepared.observed_at_ms);
            if !self.commands.bind_request_decision(&token, binding) {
                self.reject_binding(prepared.transfer, token, prepared.exploration_claim)
                    .await;
                return None;
            }
        }
        let result = self.commit_selected(selected, prepared.resources, prepared.observed_at_ms);
        if result == CommitResult::Rejected {
            self.reject_commit(prepared.transfer, action, bound, prepared.exploration_claim)
                .await;
            return None;
        }
        if result == CommitResult::Committed {
            prepared
                .transfer
                .record_network_commit(prepared.resources.network_bytes);
        }
        let launched_at_ms = time::unix_time_ms();
        let action = self.downloads.launch(
            self.ctx.clone(),
            prepared.transfer,
            launched_at_ms,
            self.state.network_class(),
            prepared.exploration_claim,
        );
        if result == CommitResult::Committed {
            self.request_immediate_replan();
        }
        Some(action)
    }

    fn admit_origin(&mut self, transfer: PlannedTransfer) -> Option<AdmittedGrant> {
        let observed_at_ms = time::unix_time_ms();
        let authority = ghostr_engine::RequestAuthority::from_url(&transfer.url)?;
        let concurrency = origin_concurrency(&self.ctx.requests, &authority);
        let query = origin_admission::query(
            &transfer,
            observed_at_ms,
            concurrency,
            self.state.network_class(),
        );
        let mode = origin_admission::mode(&transfer);
        let admission =
            self.keeper
                .stats_mut()
                .origin_model_mut()
                .claim(&query, observed_at_ms, mode);
        let exploration_claim = match &admission {
            ghostr_engine::origin_model::Admission::Exploration { claim, .. } => {
                Some(claim.clone())
            }
            _ => None,
        };
        let transfer = origin_admission::apply(transfer, &admission)?;
        Some(AdmittedGrant::new(
            transfer,
            observed_at_ms,
            exploration_claim,
        ))
    }
}

fn request_resources(request: RetrievalRequest) -> ResourceCost {
    let bytes = request.immediate_network_bytes();
    ResourceCost::new(bytes, bytes, 0, 1)
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
