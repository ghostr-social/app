//! Every snapshot says how far its page got, so Dart stops guessing
//! completeness from the row count: a page in flight is `Loading`, a
//! resolved plan is `Settled`, and a failed primary query publishes a
//! revision of its own so the adapter raises a failure instead of
//! treating partial rows as an empty feed.

use crate::api::feed_runtime::{lock, SharedFeedState};
use crate::api::feed_state::FeedState;
use crate::api::feed_types::FfiFeedStage;
use crate::api::feed_updates_stream::watch_feed;
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::feed_watch_support::{next, ChannelOut};
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::plan_executor::PlanFailure;
use nostr_sdk::Keys;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

fn opened() -> (SharedFeedState, Keys) {
    (Arc::new(Mutex::new(FeedState::new())), Keys::generate())
}

#[tokio::test]
async fn a_page_in_flight_streams_as_loading_and_then_settles() {
    let (state, keys) = opened();
    let viewer = Some(keys.public_key());
    let (feed, dispatch) = lock(&state).open(FeedSpec::MainFeed { viewer });
    let open = dispatch.expect("main feeds dispatch a first page");
    let revisions = lock(&state).subscribe(feed).expect("open feeds subscribe");
    let (sender, mut updates) = mpsc::unbounded_channel();
    tokio::spawn(watch_feed(
        ChannelOut(sender),
        state.clone(),
        feed,
        revisions,
    ));

    let baseline = next(&mut updates).await;
    lock(&state).apply(&open.context, Ok(vec![video_note(&keys, "clip", 40)]));
    let loaded = next(&mut updates).await;

    assert_eq!(baseline.stage, FfiFeedStage::Loading);
    assert_eq!(loaded.stage, FfiFeedStage::Settled);
}

#[tokio::test]
async fn a_failed_first_page_publishes_a_failed_revision() {
    let (state, keys) = opened();
    let viewer = Some(keys.public_key());
    let (feed, dispatch) = lock(&state).open(FeedSpec::MainFeed { viewer });
    let open = dispatch.expect("main feeds dispatch a first page");
    let revisions = lock(&state).subscribe(feed).expect("open feeds subscribe");
    let (sender, mut updates) = mpsc::unbounded_channel();
    tokio::spawn(watch_feed(
        ChannelOut(sender),
        state.clone(),
        feed,
        revisions,
    ));

    let baseline = next(&mut updates).await;
    lock(&state).apply(
        &open.context,
        Err(PlanFailure::new("relay down".to_owned())),
    );
    let failed = next(&mut updates).await;

    assert_eq!(failed.stage, FfiFeedStage::Failed);
    assert!(failed.revision > baseline.revision);
    assert!(failed.posts.is_empty());
}

/// A spec that can never query (blank search) has nothing to wait for.
#[tokio::test]
async fn a_feed_that_dispatches_nothing_is_settled_at_once() {
    let (state, _keys) = opened();
    let (feed, dispatch) = lock(&state).open(FeedSpec::Search("   ".to_owned()));
    let revisions = lock(&state).subscribe(feed).expect("open feeds subscribe");
    let (sender, mut updates) = mpsc::unbounded_channel();
    tokio::spawn(watch_feed(
        ChannelOut(sender),
        state.clone(),
        feed,
        revisions,
    ));

    assert!(dispatch.is_none());
    assert_eq!(next(&mut updates).await.stage, FfiFeedStage::Settled);
}
