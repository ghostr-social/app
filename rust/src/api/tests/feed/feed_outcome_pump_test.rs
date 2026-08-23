//! `pump_outcomes`: retrieval outcomes leaving the scheduler land in
//! the locked feed state and wake the feed's revision watch.

use crate::api::feed::decisions::LoadMoreAction;
use crate::api::feed::state::FeedState;
use crate::api::runtime::discovery::{lock, pump_outcomes, OutcomeSinks, SharedFeedState};
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::{RetrievalOutcome, RetrievalPurpose};
use nostr_sdk::{Keys, Timestamp};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn pumped_outcomes_reach_the_feed_state() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let keys = Keys::generate();
    let viewer = Some(keys.public_key());
    let (feed, dispatch) = lock(&state).open(FeedSpec::MainFeed { viewer });
    let open = dispatch.expect("main feeds dispatch a first page");
    let mut revisions = lock(&state).subscribe(feed).expect("open feeds subscribe");

    let (sender, outcomes) = mpsc::unbounded_channel();
    let sinks = OutcomeSinks {
        state: state.clone(),
        bootstrap: test_bootstrap().0,
        candidates: None,
    };
    let pump = tokio::spawn(pump_outcomes(sinks, outcomes));
    sender
        .send(RetrievalOutcome::Started {
            context: open.context.clone(),
        })
        .expect("the pump should be listening");
    sender
        .send(RetrievalOutcome::Completed {
            context: open.context,
            result: Ok(vec![video_note(&keys, "clip", 40)]),
            cursor: Some(Timestamp::from(99)),
            complete: true,
            purpose: RetrievalPurpose::Head,
        })
        .expect("the pump should be listening");

    timeout(Duration::from_secs(5), revisions.changed())
        .await
        .expect("the landed page should tick the revision")
        .expect("the feed should stay open");
    assert_eq!(lock(&state).snapshot(feed).len(), 1);
    let action = lock(&state).load_more(feed, None).action;
    assert!(matches!(
        action,
        LoadMoreAction::Older { older_than, .. }
            if older_than == Timestamp::from(99)
    ));

    drop(sender);
    timeout(Duration::from_secs(5), pump)
        .await
        .expect("the pump should end with its channel")
        .expect("the pump task should not panic");
}
