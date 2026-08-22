use super::support::planned_transfer;
use crate::mutable_priority_queue::{ForegroundSlots, MutablePriorityQueue};
use ghostr_engine::adaptive::PreemptionAuthority;
use std::collections::HashSet;

#[test]
fn host_breadth_never_crosses_the_highest_authority_class() {
    let mut queue = MutablePriorityQueue::new();
    queue.replace(vec![
        planned_transfer("urgent", "slow.example", PreemptionAuthority::Transition),
        planned_transfer("peer", "fast.example", PreemptionAuthority::Transition),
        planned_transfer("far", "other.example", PreemptionAuthority::Speculative),
    ]);
    let active = HashSet::from(["https://slow.example".to_owned()]);

    let selected = queue.pop_for_idle_host(&active).expect("healthy work");

    assert_eq!(selected.request.chunk.post.as_str(), "peer");
    assert!(queue.pop_for_idle_host(&active).is_none());
    let selected = queue
        .pop_for_hosts(&active, ForegroundSlots::default())
        .expect("highest-authority work");
    assert_eq!(selected.request.chunk.post.as_str(), "urgent");
    assert!(queue
        .pop_for_hosts(&HashSet::new(), ForegroundSlots::default())
        .is_none());
}
