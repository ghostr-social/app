use crate::adaptive::{ResourceCost, RetrievalRequest};
use crate::PostId;

/// The exact request authorized after live origin admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutedRequest {
    pub post: PostId,
    pub source: String,
    pub request: RetrievalRequest,
    pub resources: ResourceCost,
}

impl ExecutedRequest {
    pub(super) fn has_exact_resources(&self) -> bool {
        self.resources == resources_for(self.request)
    }
}

fn resources_for(request: RetrievalRequest) -> ResourceCost {
    let bytes = request.immediate_network_bytes();
    ResourceCost::new(bytes, bytes, 0, 1)
}
