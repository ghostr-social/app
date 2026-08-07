//! An older page always publishes a revision, even when it adds no
//! rows: the *stage* went from `Loading` back to `Settled`, and that is
//! the only signal telling the pull-shaped Dart adapter the request is
//! over. Without it a spent feed costs the adapter its whole deadline
//! (rust_feed_page_reader.dart).

use crate::api::runtime::discovery::{lock, SharedFeedState};
use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedStage;
use crate::api::feed_updates_stream::watch_feed;
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::feed_watch_support::{next, ChannelOut};
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::{FeedContext, PlanFailure};
use nostr_sdk::{Event, Keys};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type Updates = mpsc::UnboundedReceiver<crate::api::feed_types::FfiFeedUpdate>;

/// A main feed whose first page has landed, watched from that point on.
async fn loaded(state: &SharedFeedState, keys: &Keys, seen: Event) -> (FeedContext, Updates) {
    let viewer = Some(keys.public_key());
    let (feed, dispatch) = lock(state).open(FeedSpec::MainFeed { viewer });
    let open = dispatch.expect("main feeds dispatch a first page");
    lock(state).apply(&open.context, Ok(vec![seen]));
    let revisions = lock(state).subscribe(feed).expect("open feeds subscribe");
    let (sender, mut updates) = mpsc::unbounded_channel();
    tokio::spawn(watch_feed(
        ChannelOut(sender),
        state.clone(),
        feed,
        revisions,
    ));
    lock(state).load_more(feed, None);
    next(&mut updates).await;
    (open.context, updates)
}

#[tokio::test]
async fn an_older_page_that_adds_nothing_still_settles() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let keys = Keys::generate();
    let seen = video_note(&keys, "seen", 50);
    let (context, mut updates) = loaded(&state, &keys, seen.clone()).await;

    lock(&state).apply(&context, Ok(vec![seen]));

    let settled = next(&mut updates).await;
    assert_eq!(settled.stage, FfiFeedStage::Settled);
    assert_eq!(settled.posts.len(), 1);
}

#[tokio::test]
async fn a_failed_older_page_settles_as_failed() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let keys = Keys::generate();
    let (context, mut updates) = loaded(&state, &keys, video_note(&keys, "seen", 50)).await;

    lock(&state).apply(&context, Err(PlanFailure::new("relay down".to_owned())));

    assert_eq!(next(&mut updates).await.stage, FfiFeedStage::Failed);
}
