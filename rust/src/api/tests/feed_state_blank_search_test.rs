//! A spec that can never produce content (blank search) opens without
//! dispatching any query and reports no further pages — Dart returns
//! an empty page without querying
//! (DiscoveryVideoSearchRepository.searchVideos on a null normalization).

use crate::api::feed_decisions::LoadMoreAction;
use crate::api::feed_state::FeedState;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::retrieval_queue::FeedContext;

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
    let _ = state.close(feed);
    let decision = state.load_more(feed, None);
    assert!(!decision.may_have_more);
    assert!(matches!(decision.action, LoadMoreAction::None));
}

#[test]
fn a_stray_blank_search_outcome_cannot_make_it_queryable() {
    let mut state = FeedState::new();
    let (feed, _) = state.open(FeedSpec::Search(" ".to_owned()));
    let context = FeedContext::for_session(format!("feed-{}", feed.0), state.session_generation());
    state.apply(&context, Ok(Vec::new()));

    let decision = state.load_more(feed, None);
    assert!(!decision.may_have_more);
    assert!(matches!(decision.action, LoadMoreAction::None));
}
