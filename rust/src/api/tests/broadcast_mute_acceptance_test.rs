//! An accepted mute list filters the active feed without a relay echo.

use crate::api::accepted_events::remember_accepted;
use crate::api::feed_runtime::{lock, OutcomeSinks, SharedFeedState};
use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::{signed_event, video_note, SignedEventFixture};
use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::discovery::event_cache::{session_event_database, EventCache};
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::{Keys, Kind};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn accepted_mute_list_filters_the_active_viewers_feed() {
    let viewer = Keys::generate();
    let blocked = Keys::generate();
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let (feed, dispatch) = lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    let (bootstrap, _probe) = test_bootstrap();
    let sinks = OutcomeSinks {
        state: state.clone(),
        bootstrap,
        candidates: None,
    };
    let cache = EventCache::new(Arc::new(session_event_database(16)));
    let mute = signed_event(SignedEventFixture {
        keys: &viewer,
        kind: Kind::MuteList,
        content: "",
        tags: vec![vec!["p".to_owned(), blocked.public_key().to_hex()]],
        created_at: 20,
    });

    remember_accepted(&cache, &sinks, &mute).await;
    lock(&state).apply(
        &dispatch.expect("main feed").context,
        Ok(vec![video_note(&blocked, "hidden", 10)]),
    );

    assert!(lock(&state).snapshot(feed).is_empty());
}
