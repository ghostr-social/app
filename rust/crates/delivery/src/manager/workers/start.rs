use super::DownloadWorkers;
use crate::chunk::cancel::cancel_pair;
use crate::manager::admission::origin_key;
use crate::manager::inflight::ActionRegistration;
use crate::manager::plan::PlannedTransfer;
use crate::manager::transfers::{spawn_chunk, ChunkLaunch, TransferContext};
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::ActionId;
use ghostr_partial_store::partial_range_store::StoreAction;

#[must_use = "prepared transfers must be launched or released"]
pub(crate) struct PreparedTransfer {
    attempt: crate::manager::inflight::ChunkAttempt,
    priority: RangeRequest,
    retrieval: RetrievalRequest,
    host: String,
    committed_until_ms: u64,
    url: String,
    store_action: StoreAction,
    committed_network_bytes: Option<u64>,
}

impl PreparedTransfer {
    pub(crate) fn action(&self) -> ActionId {
        self.attempt.id()
    }

    pub(crate) fn record_network_commit(&mut self, bytes: u64) {
        debug_assert_eq!(
            bytes,
            self.retrieval.immediate_network_bytes(),
            "network commitment must match the prepared retrieval"
        );
        self.committed_network_bytes = Some(bytes);
    }

    pub(crate) async fn release(
        self,
        store: &ghostr_partial_store::partial_range_store::PartialRangeStore,
    ) {
        store.release_action(&self.store_action).await;
    }
}

impl DownloadWorkers {
    pub(crate) async fn prepare(
        &mut self,
        ctx: &TransferContext,
        transfer: PlannedTransfer,
    ) -> anyhow::Result<PreparedTransfer> {
        let host = origin_key(&transfer.url);
        let attempt = self.active.next_attempt_with_profile(
            transfer.request.chunk.clone(),
            transfer.identity,
            transfer.profile,
        );
        let store_action = ctx
            .store
            .reserve_action(
                attempt.identity(),
                attempt.id().value(),
                transfer.retrieval.immediate_network_bytes(),
            )
            .await?;
        Ok(PreparedTransfer {
            attempt,
            priority: transfer.request,
            retrieval: transfer.retrieval,
            host,
            committed_until_ms: transfer.commitment_until_ms,
            url: transfer.url,
            store_action,
            committed_network_bytes: None,
        })
    }

    pub(crate) fn launch(
        &mut self,
        ctx: TransferContext,
        prepared: PreparedTransfer,
        launched_at_ms: u64,
        network_class: ghostr_engine::origin_model::NetworkClass,
        exploration_claim: Option<ghostr_engine::origin_model::ExplorationClaim>,
    ) -> ActionId {
        let action = prepared.action();
        let priority = prepared.priority.authority;
        let (handle, token) = cancel_pair();
        self.active.insert_action(ActionRegistration {
            attempt: &prepared.attempt,
            priority: prepared.priority,
            retrieval: prepared.retrieval,
            host: prepared.host,
            committed_until_ms: prepared.committed_until_ms,
            launched_at_ms,
            handle,
            store_action: Some(prepared.store_action.clone()),
            committed_network_bytes: prepared.committed_network_bytes,
            exploration_claim,
        });
        spawn_chunk(ChunkLaunch {
            context: ctx,
            attempt: prepared.attempt,
            url: prepared.url,
            retrieval: prepared.retrieval,
            priority,
            token,
            action: prepared.store_action,
            network_class,
        });
        action
    }
}
