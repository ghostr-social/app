use crate::delivery_events::{DecisionToken, RequestDecisionBinding};
use crate::manager::plan::PlannedTransfer;
use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use crate::manager::workers::PreparedTransfer;
use crate::manager::{time, DeliveryWorker};
use ghostr_engine::adaptive::{ExecutedRequest, ResourceCost, RetrievalRequest};
use ghostr_engine::origin_model::AdmissionClaim;

mod admitted;
use admitted::AdmittedGrant;
mod disposition;
use disposition::GrantRejection;
mod origin;
mod rejection;

#[cfg(test)]
#[path = "grant/immediate_resources_test.rs"]
mod immediate_resources_test;
#[cfg(test)]
#[path = "grant/rejection_disposition_test.rs"]
mod rejection_disposition_test;

struct PreparedGrant {
    transfer: PreparedTransfer,
    executed: ExecutedRequest,
    resources: ResourceCost,
    observed_at_ms: u64,
    admission_claim: Option<AdmissionClaim>,
}

impl DeliveryWorker {
    pub(super) async fn grant(
        &mut self,
        transfer: PlannedTransfer,
        decision: &mut Option<DecisionToken>,
        selected: &mut Option<SelectedCommit>,
    ) -> Option<ghostr_engine::ActionId> {
        let admitted = match self.prepare_admission(transfer) {
            Ok(admitted) => admitted,
            Err(rejection) => {
                self.reject_selection(decision, rejection);
                return None;
            }
        };
        self.prepare_grant(admitted, decision, selected).await
    }

    fn prepare_admission(
        &mut self,
        transfer: PlannedTransfer,
    ) -> Result<AdmittedGrant, GrantRejection> {
        self.check_grant_eligibility(&transfer)?;
        self.admit_origin(transfer)
    }

    fn check_grant_eligibility(&self, transfer: &PlannedTransfer) -> Result<(), GrantRejection> {
        let post = &transfer.request.chunk.post;
        if self.downloads.contains_transfer(transfer) {
            return Err(GrantRejection::Duplicate);
        }
        if self.retry.is_cooling(post) {
            return Err(GrantRejection::RetryCooling);
        }
        if self.pressure.is_parked() {
            return Err(GrantRejection::StorePressure);
        }
        Ok(())
    }

    async fn prepare_grant(
        &mut self,
        admitted: AdmittedGrant,
        decision: &mut Option<DecisionToken>,
        selected: &mut Option<SelectedCommit>,
    ) -> Option<ghostr_engine::ActionId> {
        let AdmittedGrant {
            transfer,
            executed,
            resources,
            observed_at_ms,
            admission_claim,
        } = admitted;
        let post = transfer.request.chunk.post.clone();
        match self.downloads.prepare(&self.ctx, transfer).await {
            Ok(transfer) => {
                self.bind_and_launch(
                    PreparedGrant {
                        transfer,
                        executed,
                        resources,
                        observed_at_ms,
                        admission_claim,
                    },
                    decision,
                    selected,
                )
                .await
            }
            Err(error) => {
                self.reject_grant(&post, &error, decision.take(), admission_claim);
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
                self.reject_binding(prepared.transfer, token, prepared.admission_claim)
                    .await;
                return None;
            }
        }
        let result = self.commit_selected(selected, prepared.resources, prepared.observed_at_ms);
        if result == CommitResult::Rejected {
            self.reject_commit(prepared.transfer, action, bound, prepared.admission_claim)
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
            prepared.admission_claim,
        );
        if result == CommitResult::Committed {
            self.request_immediate_replan();
        }
        Some(action)
    }
}

fn request_resources(request: RetrievalRequest) -> ResourceCost {
    let bytes = request.immediate_network_bytes();
    ResourceCost::new(bytes, bytes, 0, 1)
}
