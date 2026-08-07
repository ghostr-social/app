use crate::api::feed::decisions::LoadMoreAction;
use crate::api::feed::state::FeedState;
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::Timestamp;

#[test]
fn an_empty_search_can_refresh_or_honor_an_explicit_cursor() {
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::Search("ghost".to_owned()));
    state.apply(&open.expect("search dispatch").context, Ok(Vec::new()));

    let context = match state.load_more(feed, None).action {
        LoadMoreAction::Reopen(open) => open.context,
        other => panic!("expected a reopen dispatch, got {other:?}"),
    };
    state.apply(&context, Ok(Vec::new()));
    assert!(matches!(
        state.load_more(feed, Some(Timestamp::from(10))).action,
        LoadMoreAction::Older { older_than, .. }
            if older_than == Timestamp::from(10)
    ));
}
