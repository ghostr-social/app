//! Exhaustion parity: an empty older page ends a canonical feed
//! (filtered_video_feed_repository.dart `_nextCursor` null), while a
//! failed older page keeps the cursor for the next swipe (`failLoad`).

use crate::api::feed_decisions::LoadMoreAction;
use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::plan_executor::PlanFailure;
use nostr_sdk::Keys;

#[test]
fn an_empty_older_page_exhausts_a_profile_feed() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, dispatch) = state.open(FeedSpec::Profile(keys.public_key()));
    let open = dispatch.expect("profile feeds dispatch a first page");
    state.apply(&open.context, Ok(vec![video_note(&keys, "only", 40)]));

    assert!(state.load_more(feed, None).may_have_more);
    state.apply(&open.context, Ok(Vec::new()));

    let after = state.load_more(feed, None);
    assert!(!after.may_have_more);
    assert!(matches!(after.action, LoadMoreAction::None));
}

#[test]
fn a_failed_older_page_keeps_the_cursor_for_a_retry() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, dispatch) = state.open(FeedSpec::MainFeed { viewer: Some(keys.public_key()) });
    let open = dispatch.expect("main feeds dispatch a first page");
    state.apply(&open.context, Ok(vec![video_note(&keys, "only", 40)]));

    assert!(state.load_more(feed, None).may_have_more);
    state.apply(&open.context, Err(PlanFailure::new("relay down")));

    let retry = state.load_more(feed, None);
    assert!(retry.may_have_more);
    assert!(matches!(retry.action, LoadMoreAction::Older { .. }));
}
