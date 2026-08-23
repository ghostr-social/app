use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedStage;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::RetrievalPurpose;
use nostr_sdk::Keys;

#[test]
fn empty_head_hunt_settles_and_notifies_without_dropping_rows() {
    let keys = Keys::generate();
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::Search("ghost".to_owned()));
    let context = open.expect("search opens").context;
    state.apply(&context, Ok(vec![video_note(&keys, "existing", 40)]));
    state.apply_started(&context);
    let revisions = state.subscribe(feed).expect("feed subscription");

    state.apply_retrieval(&context, Ok(Vec::new()), None, RetrievalPurpose::Head, true);

    assert!(revisions.has_changed().expect("revision sender alive"));
    assert_eq!(state.stage(feed), FfiFeedStage::Settled);
    assert_eq!(state.snapshot(feed).len(), 1);
}
