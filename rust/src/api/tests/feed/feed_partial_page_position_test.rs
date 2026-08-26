use crate::api::feed::decisions::LoadMoreAction;
use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedStage;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::RetrievalPurpose;
use nostr_sdk::{Keys, Timestamp};

#[test]
fn incomplete_first_page_stays_reopenable_and_is_replaced_when_settled() {
    let keys = Keys::generate();
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::MainFeed {
        viewer: Some(keys.public_key()),
    });
    let context = open.expect("test fixture precondition must hold").context;
    state.apply_retrieval(
        &context,
        Ok(vec![video_note(&keys, "partial", 40)]),
        None,
        RetrievalPurpose::Head,
        false,
    );

    assert!(matches!(
        state.load_more(feed, None).action,
        LoadMoreAction::Reopen(_)
    ));
    state.apply_retrieval(
        &context,
        Ok(vec![video_note(&keys, "settled", 50)]),
        Some(Timestamp::from(49)),
        RetrievalPurpose::Head,
        true,
    );
    let rows = state.snapshot(feed);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].created_at, 50);
}

#[test]
fn incomplete_older_page_keeps_the_last_authoritative_cursor() {
    let keys = Keys::generate();
    let older_keys = Keys::generate();
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::MainFeed {
        viewer: Some(keys.public_key()),
    });
    let context = open.expect("test fixture precondition must hold").context;
    state.apply_retrieval(
        &context,
        Ok(vec![video_note(&keys, "head", 100)]),
        Some(Timestamp::from(99)),
        RetrievalPurpose::Head,
        true,
    );
    state.load_more(feed, None);
    state.apply_retrieval(
        &context,
        Ok(vec![video_note(&older_keys, "partial-older", 50)]),
        Some(Timestamp::from(49)),
        RetrievalPurpose::Older,
        false,
    );

    assert_eq!(state.snapshot(feed).len(), 2);
    assert_eq!(state.stage(feed), FfiFeedStage::Failed);
    assert!(matches!(
        state.load_more(feed, None).action,
        LoadMoreAction::Older { older_than, .. } if older_than == Timestamp::from(99)
    ));
}
