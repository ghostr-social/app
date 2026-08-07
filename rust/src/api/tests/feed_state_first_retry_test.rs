//! A failed first page is retryable: the next load-more re-dispatches
//! the feed's opening query instead of paging past nothing, and while
//! any first page is pending no duplicate dispatch leaves.

use crate::api::feed_decisions::LoadMoreAction;
use crate::api::feed_state::FeedState;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::retrieval_types::PlanFailure;
use nostr_sdk::Keys;

#[test]
fn load_more_while_the_first_page_is_pending_waits() {
    let mut state = FeedState::new();
    let viewer = Some(Keys::generate().public_key());
    let (feed, dispatch) = state.open(FeedSpec::MainFeed { viewer });
    assert!(dispatch.is_some());

    let decision = state.load_more(feed, None);
    assert!(decision.may_have_more);
    assert!(matches!(decision.action, LoadMoreAction::None));
}

#[test]
fn a_failed_first_page_is_reopened_by_the_next_load_more() {
    let mut state = FeedState::new();
    let viewer = Some(Keys::generate().public_key());
    let (feed, dispatch) = state.open(FeedSpec::MainFeed { viewer });
    let open = dispatch.expect("main feeds dispatch a first page");
    state.apply(&open.context, Err(PlanFailure::new("relay down")));

    let retry = state.load_more(feed, None);
    assert!(retry.may_have_more);
    match retry.action {
        LoadMoreAction::Reopen(reopen) => {
            assert_eq!(reopen.context, open.context);
            assert_eq!(reopen.request, open.request);
        }
        other => panic!("expected a reopen dispatch, got {other:?}"),
    }
}
