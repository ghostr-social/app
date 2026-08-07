//! Takeout order follows the priority class, then submission order.

use crate::retrieval_queue::RetrievalQueue;
use crate::retrieval_types::{FeedContext, RetrievalPriority, RetrievalRequest};

fn request(context: &str, priority: RetrievalPriority) -> RetrievalRequest {
    RetrievalRequest {
        context: FeedContext::new(context),
        priority,
    }
}

#[test]
fn urgent_classes_leave_first_and_classes_stay_fifo() {
    let mut queue = RetrievalQueue::new();
    queue.push(request("a", RetrievalPriority::Background), 1);
    queue.push(request("b", RetrievalPriority::Interactive), 2);
    queue.push(request("c", RetrievalPriority::Enrichment), 3);
    queue.push(request("d", RetrievalPriority::Interactive), 4);
    queue.push(request("e", RetrievalPriority::Background), 5);

    let order: Vec<i32> =
        std::iter::from_fn(|| queue.take_next().map(|(_, payload)| payload)).collect();

    assert_eq!(order, vec![2, 4, 3, 1, 5]);
}
