//! Queue state for an unknown context never pretends a feed is open.

use crate::control_loop::FeedQueryState;
use crate::retrieval_queue::FeedContext;
use crate::scheduler_feeds::FeedBook;

#[test]
fn queued_unknown_context_is_busy_but_closed() {
    let feeds = FeedBook::default();

    let state = feeds.query_state(&FeedContext::new("missing"), true);

    assert_eq!(
        state,
        FeedQueryState {
            busy: true,
            ..FeedQueryState::default()
        }
    );
}
