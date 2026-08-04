//! A spec that can never produce content (blank search) opens without
//! dispatching any query and reports no further pages — Dart returns
//! an empty page without querying
//! (DiscoveryVideoSearchRepository.searchVideos on a null normalization).

use crate::api::feed_decisions::LoadMoreAction;
use crate::api::feed_state::FeedState;
use crate::discovery::feed_spec::FeedSpec;

#[test]
fn blank_searches_never_query_and_never_have_more() {
    let mut state = FeedState::new();
    let (feed, dispatch) = state.open(FeedSpec::Search("   ".to_owned()));
    assert!(dispatch.is_none(), "a blank search must not query relays");
    assert!(state.snapshot(feed).is_empty());

    let decision = state.load_more(feed, None);
    assert!(!decision.may_have_more);
    assert!(matches!(decision.action, LoadMoreAction::None));
}

#[test]
fn unknown_feeds_have_no_more_pages() {
    let mut state = FeedState::new();
    let (feed, _) = state.open(FeedSpec::Search("x".to_owned()));
    state.close(feed);
    let decision = state.load_more(feed, None);
    assert!(!decision.may_have_more);
    assert!(matches!(decision.action, LoadMoreAction::None));
}
