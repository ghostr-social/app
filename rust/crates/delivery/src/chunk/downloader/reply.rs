use super::ChunkSpec;
use crate::chunk::response::ResponseReply;
use ghostr_engine::adaptive::RetrievalRequest;

pub(super) fn body_spec<'a>(spec: &ChunkSpec<'a>, request: RetrievalRequest) -> ChunkSpec<'a> {
    ChunkSpec {
        client: spec.client,
        url: spec.url,
        request,
        continuation: spec.continuation,
        timeouts: spec.timeouts,
    }
}

pub(super) fn total(reply: &ResponseReply, full_length: Option<u64>) -> Option<u64> {
    match reply {
        ResponseReply::Partial { total, .. } => *total,
        ResponseReply::Body { .. } | ResponseReply::Ignored { .. } => full_length,
    }
}
