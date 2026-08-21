use super::support::planned_transfer;
use crate::manager::admission::origin_key;
use crate::mutable_priority_queue::MutablePriorityQueue;
use ghostr_engine::adaptive::PreemptionAuthority;
use std::collections::HashSet;

#[test]
fn queue_uses_canonical_request_authority_for_origin_breadth() {
    let same = planned_transfer("same", "example.com", PreemptionAuthority::Transition);
    let mut other = planned_transfer("other", "example.com", PreemptionAuthority::Transition);
    other.url = "http://example.com/other.mp4".to_owned();
    let mut queue = MutablePriorityQueue::new();
    queue.replace(vec![same, other]);
    let active = HashSet::from([origin_key("https://EXAMPLE.com:443/active.mp4")]);

    let selected = queue.pop_for_idle_host(&active).expect("distinct scheme");

    assert_eq!(selected.request.chunk.post.as_str(), "other");
}
