use super::support::transfer_identity;
use crate::manager::plan::PlannedTransfer;
use crate::mutable_priority_queue::MutablePriorityQueue;
use ghostr_engine::scoring::ChunkRequest;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::HashSet;

#[test]
fn an_idle_host_is_selected_before_more_work_for_an_active_host() {
    let mut queue = MutablePriorityQueue::new();
    queue.replace(vec![
        transfer("urgent", "slow.example"),
        transfer("ahead", "fast.example"),
    ]);
    let active = HashSet::from(["slow.example".to_owned()]);

    let selected = queue.pop_for_hosts(&active).expect("healthy work");

    assert_eq!(selected.request.chunk.post.as_str(), "ahead");
}

fn transfer(post: &str, host: &str) -> PlannedTransfer {
    let post = PostId::new(post);
    let url = format!("https://{host}/{}.mp4", post.as_str());
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: ChunkRequest {
            chunk: ChunkId {
                post,
                range: ByteRange::new(0, 4),
            },
            tier: Tier::T2Startability,
            score: 1.0,
        },
        url,
    }
}
