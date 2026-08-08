//! A focused context's work leaves the queue first, regardless of
//! priority class, and refocusing reorders already-queued work.

use crate::retrieval_types::{FeedContext, RetrievalPriority, RetrievalRequest};
use crate::scheduler::queue::RetrievalQueue;

fn request(context: &str, priority: RetrievalPriority) -> RetrievalRequest {
    RetrievalRequest {
        context: FeedContext::for_session(
            context,
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority,
    }
}

#[test]
fn focused_background_work_beats_interactive_work() {
    let mut queue = RetrievalQueue::new();
    queue.push(request("feed", RetrievalPriority::Interactive), 1);
    queue.push(request("discover", RetrievalPriority::Background), 2);

    queue.focus(FeedContext::for_session(
        "discover",
        crate::session_generation::SessionGeneration::initial(),
    ));

    assert_eq!(queue.take_next().map(|(_, p)| p), Some(2));
    assert_eq!(queue.take_next().map(|(_, p)| p), Some(1));
}

#[test]
fn refocusing_reorders_queued_work() {
    let mut queue = RetrievalQueue::new();
    queue.push(request("search", RetrievalPriority::Background), 1);
    queue.push(request("feed", RetrievalPriority::Background), 2);
    queue.focus(FeedContext::for_session(
        "search",
        crate::session_generation::SessionGeneration::initial(),
    ));

    queue.focus(FeedContext::for_session(
        "feed",
        crate::session_generation::SessionGeneration::initial(),
    ));

    assert_eq!(queue.take_next().map(|(_, p)| p), Some(2));
    assert_eq!(queue.take_next().map(|(_, p)| p), Some(1));
}

#[test]
fn context_exposes_its_stable_name() {
    let context = FeedContext::for_session(
        "discover",
        crate::session_generation::SessionGeneration::initial(),
    );

    assert_eq!(context.as_str(), "discover");
}
