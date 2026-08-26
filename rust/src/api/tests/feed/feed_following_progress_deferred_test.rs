use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedStage;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::RetrievalPurpose;
use nostr_sdk::Keys;

#[test]
fn following_progress_waits_for_page_deletion_checks() {
    let creator = Keys::generate();
    let event = video_note(&creator, "direct", 20);
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::Following {
        viewer: None,
        follows: vec![creator.public_key()],
    });
    let context = open.expect("following dispatch").context;

    state.apply_progress(&context, &event);

    assert_eq!(state.stage(feed), FfiFeedStage::Loading);
    assert!(state.snapshot(feed).is_empty());
    state.apply(&context, Ok(vec![event]));
    assert_eq!(state.snapshot(feed).len(), 1);
}

#[test]
fn incomplete_following_page_is_visible_without_settling_its_cursor() {
    let creator = Keys::generate();
    let event = video_note(&creator, "partial", 20);
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::Following {
        viewer: None,
        follows: vec![creator.public_key()],
    });
    let context = open.expect("following dispatch").context;

    state.apply_retrieval(
        &context,
        Ok(vec![event]),
        None,
        RetrievalPurpose::Head,
        false,
    );

    assert_eq!(state.snapshot(feed).len(), 1);
    assert_eq!(state.stage(feed), FfiFeedStage::Failed);
}
