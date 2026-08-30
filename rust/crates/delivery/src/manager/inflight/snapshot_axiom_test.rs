use super::*;

impl ActiveAction {
    pub(crate) fn range(
        chunk: ghostr_engine::ChunkId,
        identity: TransferIdentity,
        committed_until_ms: u64,
    ) -> Self {
        Self::range_with_action(ActionId::new(1), chunk, identity, committed_until_ms)
    }
    pub(crate) fn range_with_action(
        action_id: ActionId,
        chunk: ghostr_engine::ChunkId,
        identity: TransferIdentity,
        committed_until_ms: u64,
    ) -> Self {
        let request = RetrievalRequest::FetchRange {
            bytes: chunk.range,
            promotion: None,
        };
        Self {
            action_id,
            post: chunk.post,
            identity,
            request,
            effective_bytes: chunk.range,
            reserved_storage_bytes: chunk.range.len(),
            promotion_opportunity: None,
            committed_until_ms,
            launched_at_ms: 0,
            io_finished: false,
            cancelling: false,
            hedged: false,
        }
    }

    pub(crate) fn cancelling_for_test(mut self) -> Self {
        self.cancelling = true;
        self
    }

    pub(crate) fn hedged_for_test(mut self) -> Self {
        self.hedged = true;
        self
    }

    pub(crate) fn effective_bytes_for_test(mut self, bytes: ByteRange) -> Self {
        self.effective_bytes = bytes;
        self
    }
}
