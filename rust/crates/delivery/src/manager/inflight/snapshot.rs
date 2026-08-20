use super::InFlightChunks;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ActionId, ByteRange, PostId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveAction {
    action_id: ActionId,
    post: PostId,
    identity: TransferIdentity,
    request: RetrievalRequest,
    effective_bytes: ByteRange,
    reserved_storage_bytes: u64,
    committed_until_ms: u64,
    cancelling: bool,
}

impl ActiveAction {
    #[cfg(test)]
    pub(crate) fn range(
        chunk: ghostr_engine::ChunkId,
        identity: TransferIdentity,
        committed_until_ms: u64,
    ) -> Self {
        let request = RetrievalRequest::FetchRange {
            bytes: chunk.range,
            promotion: None,
        };
        Self {
            action_id: ActionId::new(1),
            post: chunk.post,
            identity,
            request,
            effective_bytes: chunk.range,
            reserved_storage_bytes: chunk.range.len(),
            committed_until_ms,
            cancelling: false,
        }
    }

    pub(crate) fn post(&self) -> &PostId {
        &self.post
    }

    pub(crate) fn action_id(&self) -> ActionId {
        self.action_id
    }

    pub(crate) fn request(&self) -> RetrievalRequest {
        self.request
    }

    pub(crate) fn effective_bytes(&self) -> ByteRange {
        self.effective_bytes
    }

    pub(crate) fn reserved_storage_bytes(&self) -> u64 {
        self.reserved_storage_bytes
    }

    pub(crate) fn cancelling(&self) -> bool {
        self.cancelling
    }

    pub(crate) fn identity(&self) -> &TransferIdentity {
        &self.identity
    }

    pub(crate) fn committed_until_ms(&self) -> u64 {
        self.committed_until_ms
    }
}

impl InFlightChunks {
    pub(crate) fn actions(&self) -> Vec<ActiveAction> {
        self.transfers
            .values()
            .map(|active| ActiveAction {
                action_id: active.action_id,
                post: active.chunk.post.clone(),
                identity: active.identity.clone(),
                request: active.effective_request,
                effective_bytes: active.effective_bytes,
                reserved_storage_bytes: active.reserved_storage_bytes,
                committed_until_ms: active.committed_until_ms,
                cancelling: active.cancelling,
            })
            .collect()
    }

    pub(crate) fn contains_identity(&self, identity: &TransferIdentity) -> bool {
        self.transfers
            .values()
            .any(|active| &active.identity == identity)
    }
}
