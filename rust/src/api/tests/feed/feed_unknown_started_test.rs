use crate::api::feed::state::FeedState;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::FeedContext;

#[test]
fn an_unknown_retrieval_start_cannot_mutate_an_open_feed() {
    let mut state = FeedState::new();
    let (feed, _) = state.open(FeedSpec::Search("known".to_owned()));
    let unknown = FeedContext::for_session("feed-missing", state.session_generation());
    let stage = state.stage(feed);
    let revisions = state.subscribe(feed).expect("open feeds subscribe");

    state.apply_started(&unknown);

    assert!(!revisions.has_changed().expect("the feed should stay open"));
    assert_eq!(state.stage(feed), stage);
    assert!(state.snapshot(feed).is_empty());
}
