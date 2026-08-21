use super::{ChunkSpec, ResponseWriteMode};
use crate::chunk::response::ResponseReply;
use ghostr_engine::adaptive::RetrievalRequest;

pub(super) fn body_spec<'a>(spec: &ChunkSpec<'a>, request: RetrievalRequest) -> ChunkSpec<'a> {
    ChunkSpec {
        requests: spec.requests,
        url: spec.url,
        request,
        priority: spec.priority,
        continuation: spec.continuation,
        timeouts: spec.timeouts,
    }
}

pub(super) fn range_spec<'a>(
    spec: &ChunkSpec<'a>,
    range: ghostr_engine::ByteRange,
) -> ChunkSpec<'a> {
    body_spec(
        spec,
        RetrievalRequest::FetchRange {
            bytes: range,
            promotion: None,
        },
    )
}

pub(super) fn response_mode(request: RetrievalRequest) -> ResponseWriteMode {
    match request {
        RetrievalRequest::FetchWhole { contract, .. } => {
            ResponseWriteMode::SingleResponse(contract)
        }
        RetrievalRequest::FetchRange { .. } => ResponseWriteMode::Sparse,
    }
}

pub(super) fn total(reply: &ResponseReply, full_length: Option<u64>) -> Option<u64> {
    match reply {
        ResponseReply::Partial { total, .. } => *total,
        ResponseReply::Body { .. } | ResponseReply::Ignored { .. } => full_length,
    }
}
