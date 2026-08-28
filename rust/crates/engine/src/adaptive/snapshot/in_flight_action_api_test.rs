use super::InFlightAction;
use crate::adaptive::RetrievalRequest;
use crate::{ActionId, ByteRange};

impl InFlightAction {
    pub(crate) fn range(
        action_id: ActionId,
        bytes: ByteRange,
        source: impl Into<String>,
        committed_until_ms: u64,
        identity_current: bool,
    ) -> Self {
        Self {
            action_id,
            request: RetrievalRequest::FetchRange {
                bytes,
                promotion: None,
            },
            effective_bytes: bytes,
            reserved_storage_bytes: bytes.len(),
            promotion_opportunity: None,
            source: source.into(),
            committed_until_ms,
            identity_current,
            cancelling: false,
        }
    }
}
