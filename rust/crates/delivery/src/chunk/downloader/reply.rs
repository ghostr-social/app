use super::ChunkSpec;
use crate::chunk::response::RangeReply;
use ghostr_engine::ByteRange;

pub(super) fn full_body_spec<'a>(spec: &ChunkSpec<'a>, full_length: Option<u64>) -> ChunkSpec<'a> {
    let end = full_length
        .filter(|length| *length > 0)
        .map_or(spec.range.end, |length| length.min(spec.range.end));
    ChunkSpec {
        client: spec.client,
        url: spec.url,
        range: ByteRange::new(spec.range.start, end),
        continuation: spec.continuation,
        timeouts: spec.timeouts,
    }
}

pub(super) fn total(reply: &RangeReply, full_length: Option<u64>) -> Option<u64> {
    match reply {
        RangeReply::Partial { total, .. } => *total,
        RangeReply::FullBody | RangeReply::Ignored => full_length,
    }
}
