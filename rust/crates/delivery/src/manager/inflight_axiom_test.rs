use super::*;

impl InFlightChunks {
    pub(crate) fn insert(
        &mut self,
        attempt: &ChunkAttempt,
        priority: ghostr_engine::scheduling::RangeRequest,
        host: String,
        committed_until_ms: u64,
        handle: crate::chunk::cancel::CancelHandle,
    ) {
        let retrieval = ghostr_engine::adaptive::RetrievalRequest::FetchRange {
            bytes: priority.chunk.range,
            promotion: None,
        };
        self.insert_action(ActionRegistration {
            attempt,
            priority,
            retrieval,
            host,
            committed_until_ms,
            launched_at_ms: 0,
            handle,
            store_action: None,
            committed_network_bytes: None,
            exploration_claim: None,
        });
    }
    pub(crate) fn finish(&mut self, attempt: &ChunkAttempt) -> CompletionStatus {
        self.finish_with_resources(attempt).status()
    }
}
