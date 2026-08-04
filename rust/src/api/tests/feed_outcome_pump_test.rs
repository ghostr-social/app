//! `pump_outcomes`: retrieval outcomes leaving the scheduler land in
//! the locked feed state and wake the feed's revision watch.

use crate::api::feed_runtime::{lock, pump_outcomes, SharedFeedState};
use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::discovery_scheduler::RetrievalOutcome;
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::Keys;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn pumped_outcomes_reach_the_feed_state() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let keys = Keys::generate();
    let (feed, dispatch) = lock(&state).open(FeedSpec::MainFeed { viewer: keys.public_key() });
    let open = dispatch.expect("main feeds dispatch a first page");
    let mut revisions = lock(&state).subscribe(feed).expect("open feeds subscribe");

    let (sender, outcomes) = mpsc::unbounded_channel();
    let pump = tokio::spawn(pump_outcomes(state.clone(), outcomes));
    sender
        .send(RetrievalOutcome {
            context: open.context,
            result: Ok(vec![video_note(&keys, "clip", 40)]),
        })
        .expect("the pump should be listening");

    timeout(Duration::from_secs(5), revisions.changed())
        .await
        .expect("the landed page should tick the revision")
        .expect("the feed should stay open");
    assert_eq!(lock(&state).snapshot(feed).len(), 1);

    drop(sender);
    timeout(Duration::from_secs(5), pump)
        .await
        .expect("the pump should end with its channel")
        .expect("the pump task should not panic");
}
