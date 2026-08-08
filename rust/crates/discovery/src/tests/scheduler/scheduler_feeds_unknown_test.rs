//! Queue state for an unknown context never pretends a feed is open.

use crate::retrieval_types::FeedContext;
use crate::scheduler::control::FeedQueryState;
use crate::scheduler::feeds::FeedBook;

#[test]
fn queued_unknown_context_is_busy_but_closed() {
    let feeds = FeedBook::default();

    let context = FeedContext::for_session(
        "missing",
        crate::session_generation::SessionGeneration::initial(),
    );
    let state = feeds.query_state(&context, true);

    assert_eq!(
        state,
        FeedQueryState {
            busy: true,
            ..FeedQueryState::default()
        }
    );
}
