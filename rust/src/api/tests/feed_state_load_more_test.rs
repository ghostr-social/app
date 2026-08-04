//! Load-more gating: one older request in flight at a time, the claimed
//! cursor rides the dispatch, and an explicit FFI cursor wins over the
//! tracked one (scheduler LoadMore parity).

use crate::api::feed_decisions::{LoadMoreAction, OpenDispatch};
use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::{Keys, Timestamp};

fn loaded_feed(state: &mut FeedState, keys: &Keys) -> (crate::discovery::feed_store::FeedId, OpenDispatch) {
    let (feed, dispatch) = state.open(FeedSpec::MainFeed { viewer: keys.public_key() });
    let open = dispatch.expect("main feeds dispatch a first page");
    state.apply(&open.context, Ok(vec![video_note(keys, "a", 30), video_note(keys, "b", 40)]));
    (feed, open)
}

#[test]
fn load_more_claims_the_cursor_below_the_oldest_visible_post() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, open) = loaded_feed(&mut state, &keys);

    let decision = state.load_more(feed, None);
    assert!(decision.may_have_more);
    match decision.action {
        LoadMoreAction::Older { context, older_than } => {
            assert_eq!(context, open.context);
            assert_eq!(older_than, Timestamp::from(29));
        }
        other => panic!("expected an older-page dispatch, got {other:?}"),
    }
}

#[test]
fn only_one_older_request_flies_at_a_time() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, _) = loaded_feed(&mut state, &keys);

    assert!(state.load_more(feed, None).may_have_more);
    let second = state.load_more(feed, None);
    assert!(second.may_have_more, "an in-flight load still promises more");
    assert!(matches!(second.action, LoadMoreAction::None));
}

#[test]
fn an_explicit_cursor_wins_over_the_tracked_one() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, _) = loaded_feed(&mut state, &keys);

    let decision = state.load_more(feed, Some(Timestamp::from(10)));
    match decision.action {
        LoadMoreAction::Older { older_than, .. } => assert_eq!(older_than, Timestamp::from(10)),
        other => panic!("expected an older-page dispatch, got {other:?}"),
    }
}
