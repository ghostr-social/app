use crate::api::feed_decisions::LoadMoreAction;
use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::{signed_event, SignedEventFixture};
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::{Keys, Kind, Timestamp};

#[test]
fn non_playable_search_hits_still_advance_the_scan_cursor() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, open) = state.open(FeedSpec::Search("ghost".to_owned()));
    let note = signed_event(SignedEventFixture {
        keys: &keys,
        kind: Kind::TextNote,
        content: "ghost without media",
        tags: Vec::new(),
        created_at: 100,
    });

    state.apply(&open.expect("search dispatch").context, Ok(vec![note]));
    assert!(state.snapshot(feed).is_empty());

    match state.load_more(feed, None).action {
        LoadMoreAction::Older { older_than, .. } => {
            assert_eq!(older_than, Timestamp::from(99));
        }
        other => panic!("expected an older-page dispatch, got {other:?}"),
    }
}
