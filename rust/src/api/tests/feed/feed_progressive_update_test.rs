use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedStage;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::Keys;

#[test]
fn a_progress_event_is_visible_while_the_page_keeps_loading() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, open) = state.open(FeedSpec::Search("ghost".to_owned()));
    let context = open.expect("search dispatch").context;
    let event = video_note(&keys, "early", 40);

    state.apply_progress(&context, event.clone());

    assert_eq!(state.stage(feed), FfiFeedStage::Loading);
    assert_eq!(state.snapshot(feed).len(), 1);

    state.apply(&context, Ok(vec![event]));
    assert_eq!(state.stage(feed), FfiFeedStage::Settled);
    assert_eq!(state.snapshot(feed).len(), 1);
}
