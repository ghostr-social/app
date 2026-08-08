use crate::manager::plan::PlannedTransfer;
use crate::mutable_priority_queue::MutablePriorityQueue;
use ghostr_engine::scoring::ChunkRequest;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn replacement_reprioritizes_work_and_drops_stale_entries() {
    let low = transfer("low", 1.0);
    let high = transfer("high", 2.0);
    let mut queue = MutablePriorityQueue::new();
    queue.replace(vec![low.clone(), high.clone()]);

    queue.replace(vec![high.clone()]);

    assert_eq!(queue.wanted(), [high.request.chunk.clone()].into());
    assert_eq!(
        queue.pop().expect("queued work").request.chunk.post.0,
        "high"
    );
    assert!(queue.pop().is_none());
}

fn transfer(id: &str, score: f64) -> PlannedTransfer {
    PlannedTransfer {
        request: ChunkRequest {
            chunk: ChunkId {
                post: PostId::new(id),
                range: ByteRange::new(0, 4),
            },
            tier: Tier::T2Startability,
            score,
        },
        url: format!("https://media.example/{id}.mp4"),
    }
}
