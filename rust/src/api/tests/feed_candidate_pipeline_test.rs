use crate::api::feed_runtime::{lock, pump_outcomes, OutcomeSinks, SharedFeedState};
use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::retrieval_types::RetrievalOutcome;
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use nostr_sdk::Keys;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn relay_progress_admits_a_candidate_before_page_completion() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let (feed, dispatch) = lock(&state).open(FeedSpec::Search("clip".to_owned()));
    let context = dispatch.expect("search dispatch").context;
    let (delivery, mut commands) = command_channel();
    let (sender, outcomes) = mpsc::unbounded_channel();
    let pump = tokio::spawn(pump_outcomes(
        OutcomeSinks {
            state: state.clone(),
            bootstrap: test_bootstrap().0,
            candidates: Some(delivery),
        },
        outcomes,
    ));

    sender
        .send(RetrievalOutcome::Progress {
            context,
            event: Box::new(video_note(&Keys::generate(), "early", 40)),
        })
        .expect("progress receiver");

    let command = tokio::time::timeout(Duration::from_secs(1), commands.recv())
        .await
        .expect("candidate should be immediate")
        .expect("delivery command");
    let DeliveryCommand::Candidate(candidate) = command else {
        panic!("expected candidate admission");
    };
    assert_eq!(candidate.meta.urls, ["https://cdn.example/early.mp4"]);
    assert_eq!(lock(&state).snapshot(feed).len(), 1);

    drop(sender);
    pump.await.expect("outcome pump");
}
