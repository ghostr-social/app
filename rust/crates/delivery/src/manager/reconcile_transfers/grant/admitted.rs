use super::request_resources;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::adaptive::{ExecutedRequest, ResourceCost};

pub(super) struct AdmittedGrant {
    pub(super) transfer: PlannedTransfer,
    pub(super) executed: ExecutedRequest,
    pub(super) resources: ResourceCost,
    pub(super) observed_at_ms: u64,
}

impl AdmittedGrant {
    pub(super) fn new(transfer: PlannedTransfer, observed_at_ms: u64) -> Self {
        let resources = request_resources(transfer.retrieval);
        let executed = ExecutedRequest {
            post: transfer.request.chunk.post.clone(),
            source: transfer.url.clone(),
            request: transfer.retrieval,
            resources,
        };
        Self {
            transfer,
            executed,
            resources,
            observed_at_ms,
        }
    }
}
