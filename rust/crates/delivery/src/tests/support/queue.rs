use super::{range_profile, transfer_identity};
use crate::manager::plan::PlannedTransfer;
use crate::mutable_priority_queue::MutablePriorityQueue;
use ghostr_engine::adaptive::{ControlMode, PreemptionAuthority, RetrievalRequest};
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::HashSet;

pub(crate) fn planned_transfer(
    name: &str,
    host: &str,
    authority: PreemptionAuthority,
) -> PlannedTransfer {
    let post = PostId::new(name);
    let url = format!("https://{host}/{name}.mp4");
    PlannedTransfer {
        control_mode: ControlMode::Normal,
        identity: transfer_identity(&post, &url),
        request: range_request(post, authority),
        retrieval: range_retrieval(),
        profile: range_profile(4),
        url,
        commitment_until_ms: 0,
    }
}

fn range_request(post: PostId, authority: PreemptionAuthority) -> RangeRequest {
    RangeRequest {
        chunk: ChunkId {
            post,
            range: ByteRange::new(0, 4),
        },
        authority,
        score: 1.0,
        contiguous_depth_bytes: 0,
    }
}

fn range_retrieval() -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 4),
        promotion: None,
    }
}

pub(crate) fn planned_queue(
    items: &[(&str, PreemptionAuthority)],
    host: &str,
) -> MutablePriorityQueue {
    let mut queue = MutablePriorityQueue::new();
    queue.replace(
        items
            .iter()
            .map(|(name, authority)| planned_transfer(name, host, *authority))
            .collect(),
    );
    queue
}

pub(crate) fn active_hosts(host: &str) -> HashSet<String> {
    HashSet::from([host.to_owned()])
}

pub(crate) fn transfer_posts<const N: usize>(items: &[PlannedTransfer; N]) -> [String; N] {
    core::array::from_fn(|index| items[index].request.chunk.post.as_str().to_owned())
}
