//! A focused context's work leaves the queue first, regardless of
//! priority class, and refocusing reorders already-queued work —
//! parity: `focus` in lib/core/work/retrieval_scheduler.dart.

use crate::discovery::retrieval_queue::{
    FeedContext, RetrievalPriority, RetrievalQueue, RetrievalRequest,
};

fn request(context: &str, priority: RetrievalPriority) -> RetrievalRequest {
    RetrievalRequest {
        context: FeedContext::new(context),
        priority,
    }
}

#[test]
fn focused_background_work_beats_interactive_work() {
    let mut queue = RetrievalQueue::new();
    queue.push(request("feed", RetrievalPriority::Interactive), 1);
    queue.push(request("discover", RetrievalPriority::Background), 2);

    queue.focus(FeedContext::new("discover"));

    assert_eq!(queue.take_next().map(|(_, p)| p), Some(2));
    assert_eq!(queue.take_next().map(|(_, p)| p), Some(1));
}

#[test]
fn refocusing_reorders_queued_work() {
    let mut queue = RetrievalQueue::new();
    queue.push(request("search", RetrievalPriority::Background), 1);
    queue.push(request("feed", RetrievalPriority::Background), 2);
    queue.focus(FeedContext::new("search"));

    queue.focus(FeedContext::new("feed"));

    assert_eq!(queue.take_next().map(|(_, p)| p), Some(2));
    assert_eq!(queue.take_next().map(|(_, p)| p), Some(1));
}
