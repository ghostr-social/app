//! `watch_feed`: every subscription starts with a baseline snapshot,
//! each visible-list revision streams a fresh full snapshot, and the
//! stream ends when the feed closes.

use crate::api::feed_runtime::{lock, SharedFeedState};
use crate::api::feed_state::FeedState;
use crate::api::feed_updates_stream::watch_feed;
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::feed_watch_support::{next, ChannelOut};
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::Keys;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn snapshots_stream_from_baseline_to_close() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let keys = Keys::generate();
    let viewer = Some(keys.public_key());
    let (feed, dispatch) = lock(&state).open(FeedSpec::MainFeed { viewer });
    let open = dispatch.expect("main feeds dispatch a first page");
    let revisions = lock(&state).subscribe(feed).expect("open feeds subscribe");

    let (sender, mut updates) = mpsc::unbounded_channel();
    let watcher = tokio::spawn(watch_feed(
        ChannelOut(sender),
        state.clone(),
        feed,
        revisions,
    ));

    let baseline = next(&mut updates).await;
    assert_eq!(baseline.feed_id, format!("{}", feed.0));
    assert!(baseline.posts.is_empty());

    lock(&state).apply(&open.context, Ok(vec![video_note(&keys, "clip", 40)]));
    let loaded = next(&mut updates).await;
    assert_eq!(loaded.posts.len(), 1);
    assert!(loaded.revision > baseline.revision);

    let _ = lock(&state).close(feed);
    timeout(Duration::from_secs(5), watcher)
        .await
        .expect("the watcher should end when the feed closes")
        .expect("the watcher task should not panic");
}
