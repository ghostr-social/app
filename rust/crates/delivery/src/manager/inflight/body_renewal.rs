use super::{ChunkAttempt, InFlightChunks, ResponsePhase};
use ghostr_engine::adaptive::{RetrievalRequest, REQUEST_SLICE_BYTES};

impl InFlightChunks {
    pub(crate) fn body_renewal_delta(&self, attempt: &ChunkAttempt, through: u64) -> Option<u64> {
        let active = self.transfers.get(&attempt.id())?;
        if active.identity != *attempt.identity() || active.cancelling || active.io_finished() {
            return None;
        }
        if active.response_phase != ResponsePhase::Opened { return None; }
        let RetrievalRequest::FetchWhole { contract, .. } = active.effective_request else { return None; };
        let reserved = active.committed_network_bytes.saturating_add(active.uncommitted_network_prefix_bytes);
        let delta = through.saturating_sub(reserved);
        (through <= contract.maximum_bytes() && delta <= REQUEST_SLICE_BYTES).then_some(delta)
    }

    pub(crate) fn commit_body_renewal(&mut self, attempt: &ChunkAttempt, delta: u64) {
        if let Some(active) = self.transfers.get_mut(&attempt.id()) {
            active.committed_network_bytes = active.committed_network_bytes.saturating_add(delta);
        }
    }
}
