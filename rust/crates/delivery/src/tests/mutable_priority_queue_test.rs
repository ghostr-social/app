use super::support::{range_retrieval, transfer_identity};
use crate::manager::plan::PlannedTransfer;
use crate::mutable_priority_queue::{ForegroundSlots, MutablePriorityQueue};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::HashSet;

#[test]
fn replacement_reprioritizes_work_and_drops_stale_entries() {
    let low = transfer("low", 1.0);
    let high = transfer("high", 2.0);
    let mut queue = MutablePriorityQueue::new();
    queue.replace(vec![low.clone(), high.clone()]);

    queue.replace(vec![high.clone()]);

    assert_eq!(queue.wanted(), [high.id()].into());
    assert_eq!(
        queue
            .pop_for_hosts(&HashSet::new(), ForegroundSlots::default())
            .expect("queued work")
            .request
            .chunk
            .post
            .0,
        "high"
    );
    assert!(queue
        .pop_for_hosts(&HashSet::new(), ForegroundSlots::default())
        .is_none());
}

fn transfer(id: &str, score: f64) -> PlannedTransfer {
    let post = PostId::new(id);
    let url = format!("https://media.example/{id}.mp4");
    PlannedTransfer {
        control_mode: ghostr_engine::adaptive::ControlMode::Normal,
        identity: transfer_identity(&post, &url),
        request: RangeRequest {
            chunk: ChunkId {
                post,
                range: ByteRange::new(0, 4),
            },
            authority: PreemptionAuthority::Transition,
            score,
            contiguous_depth_bytes: 0,
        },
        url,
        retrieval: range_retrieval(ByteRange::new(0, 4)),
        profile: crate::tests::support::range_profile(4),
        commitment_until_ms: 0,
    }
}
