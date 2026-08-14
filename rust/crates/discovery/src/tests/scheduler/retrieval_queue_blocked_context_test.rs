use crate::retrieval_types::{RetrievalPriority, RetrievalRequest};
use crate::scheduler::queue::RetrievalQueue;
use crate::tests::scheduler_support::context;

#[test]
fn blocked_focus_does_not_idle_an_available_worker() {
    let blocked = context("blocked");
    let eligible = context("eligible");
    let mut queue = RetrievalQueue::new();
    queue.focus(blocked.clone());
    queue.push(request(blocked.clone()), 1);
    queue.push(request(eligible.clone()), 2);

    let next = queue
        .take_next_excluding([&blocked])
        .expect("eligible work remains");

    assert_eq!(next.0.context, eligible);
    assert_eq!(next.1, 2);
    assert_eq!(queue.take_next().map(|(_, value)| value), Some(1));
}

fn request(context: crate::retrieval_types::FeedContext) -> RetrievalRequest {
    RetrievalRequest {
        context,
        priority: RetrievalPriority::Background,
    }
}
