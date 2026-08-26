use crate::adaptive::PreemptionAuthority;
use crate::ChunkId;
use core::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub struct RangeRequest {
    pub chunk: ChunkId,
    pub authority: PreemptionAuthority,
    pub score: f64,
    pub contiguous_depth_bytes: u64,
}

pub fn compare(left: &RangeRequest, right: &RangeRequest) -> Ordering {
    left.authority
        .cmp(&right.authority)
        .then_with(|| transition_depth_order(left, right))
        .then_with(|| right.score.total_cmp(&left.score))
        .then_with(|| left.chunk.post.cmp(&right.chunk.post))
        .then_with(|| left.chunk.range.start.cmp(&right.chunk.range.start))
}

fn transition_depth_order(left: &RangeRequest, right: &RangeRequest) -> Ordering {
    let both_transition = left.authority == PreemptionAuthority::Transition
        && right.authority == PreemptionAuthority::Transition;
    if both_transition {
        left.contiguous_depth_bytes
            .cmp(&right.contiguous_depth_bytes)
    } else {
        Ordering::Equal
    }
}
