//! Reset drops feed-owned account state and never reuses a feed handle.

use crate::api::feed::state::FeedState;
use crate::api::tests::feed_fixtures::{profile_event, video_note};
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::Keys;

#[tokio::test]
async fn reset_ends_streams_and_rejects_the_old_feed_context() {
    let mut state = FeedState::new();
    let author = Keys::generate();
    let (stale, dispatch) = state.open(FeedSpec::MainFeed {
        viewer: Some(author.public_key()),
    });
    let stale_context = dispatch.expect("main feed").context;
    let mut revisions = state.subscribe(stale).expect("open feed stream");
    state.apply(
        &stale_context,
        Ok(vec![
            profile_event(&author, r#"{"name":"Old Name"}"#, 10),
            video_note(&author, "old", 20),
        ]),
    );
    revisions.borrow_and_update();

    let fresh_generation = state.reset_session();

    assert!(revisions.changed().await.is_err(), "stream must end");
    state.apply(
        &stale_context,
        Ok(vec![video_note(&author, "late-old", 30)]),
    );
    let (fresh, dispatch) = state.open(FeedSpec::MainFeed {
        viewer: Some(author.public_key()),
    });
    let fresh_context = dispatch.expect("fresh feed").context;
    state.apply(&fresh_context, Ok(vec![video_note(&author, "fresh", 40)]));

    assert!(fresh.0 > stale.0, "feed ids must stay monotonic");
    assert_ne!(fresh_context.session(), stale_context.session());
    assert_eq!(fresh_context.session(), fresh_generation);
    assert!(state.snapshot(stale).is_empty());
    let rows = state.snapshot(fresh);
    assert_eq!(rows.len(), 1);
    assert_ne!(rows[0].creator.display_name, "Old Name");
}
