use crate::api::feed::state::FeedState;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::RetrievalPurpose;
use nostr_sdk::Keys;

#[test]
fn native_head_refresh_is_loading_and_keeps_chronological_rows() {
    let keys = Keys::generate();
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::Search("ghost".to_owned()));
    let context = open.expect("search opens").context;
    state.apply(&context, Ok(vec![video_note(&keys, "older.mp4", 40)]));

    state.apply_started(&context);
    assert_eq!(
        state.stage(feed),
        crate::api::feed_types::FfiFeedStage::Loading
    );
    state.apply_retrieval(
        &context,
        Ok(vec![video_note(&keys, "newer.mp4", 80)]),
        RetrievalPurpose::Head,
    );

    let rows = state.snapshot(feed);
    assert_eq!(rows.len(), 2);
    assert!(rows[0].created_at > rows[1].created_at);
}
