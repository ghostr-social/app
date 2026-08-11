use super::support::planned_transfer;
use crate::mutable_priority_queue::{ForegroundSlots, MutablePriorityQueue};
use ghostr_engine::tiers::Tier;
use std::collections::HashSet;

#[test]
fn host_breadth_never_crosses_the_head_tier() {
    let mut queue = MutablePriorityQueue::new();
    queue.replace(vec![
        planned_transfer("urgent", "slow.example", Tier::T2Startability),
        planned_transfer("peer", "fast.example", Tier::T2Startability),
        planned_transfer("far", "other.example", Tier::T4Speculative),
    ]);
    let active = HashSet::from(["slow.example".to_owned()]);

    let selected = queue
        .pop_for_hosts(&active, ForegroundSlots::default())
        .expect("healthy work");

    assert_eq!(selected.request.chunk.post.as_str(), "peer");
    assert!(queue.pop_for_idle_host(&active).is_none());
    let selected = queue
        .pop_for_hosts(&active, ForegroundSlots::default())
        .expect("head-tier work");
    assert_eq!(selected.request.chunk.post.as_str(), "urgent");
    assert!(queue
        .pop_for_hosts(&HashSet::new(), ForegroundSlots::default())
        .is_none());
}
