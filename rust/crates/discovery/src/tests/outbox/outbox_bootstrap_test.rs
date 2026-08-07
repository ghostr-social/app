//! Outbox bootstrap chases relay lists outside scheduler worker slots.
//! These calls return while the recording executor remains outstanding.

use crate::outbox::bootstrap::OutboxBootstrap;
use crate::plan_executor::PlannedRetrieval;
use crate::tests::outbox_support::{
    empty_directory, failing_executor, recording_executor, relay_list_event,
};
use crate::tests::support::{author, filter_json, AUTHOR_A};
use nostr_sdk::Keys;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

type Retrievals = mpsc::UnboundedReceiver<PlannedRetrieval>;

async fn requested_authors(retrievals: &mut Retrievals) -> serde_json::Value {
    let retrieval = timeout(Duration::from_secs(5), retrievals.recv())
        .await
        .expect("the bootstrap should start a retrieval")
        .expect("the recorder should stay alive");
    filter_json(&retrieval.plan.queries[0].filter)["authors"].clone()
}

#[tokio::test]
async fn the_viewer_is_chased_once() {
    let (executor, mut retrievals) = recording_executor();
    let (outcomes, _sink) = mpsc::unbounded_channel();
    let bootstrap = OutboxBootstrap::new(executor, empty_directory(), outcomes);

    bootstrap.viewer(author(AUTHOR_A));
    bootstrap.viewer(author(AUTHOR_A));

    assert_eq!(
        requested_authors(&mut retrievals).await,
        serde_json::json!([AUTHOR_A])
    );
    assert!(
        retrievals.try_recv().is_err(),
        "the second ask is deduplicated"
    );
}

#[tokio::test]
async fn landed_follows_are_chased_for_their_relay_lists() {
    let (executor, mut retrievals) = recording_executor();
    let (outcomes, _sink) = mpsc::unbounded_channel();
    let directory = empty_directory();
    let bootstrap = OutboxBootstrap::new(executor, directory.clone(), outcomes);

    bootstrap.track_follows(vec![author(AUTHOR_A)]).await;

    assert_eq!(
        requested_authors(&mut retrievals).await,
        serde_json::json!([AUTHOR_A])
    );
}

/// A failed startup chase releases its NIP-65 routing claim.
#[tokio::test]
async fn a_failed_chase_is_asked_again() {
    let (executor, mut retrievals) = failing_executor();
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let bootstrap = OutboxBootstrap::new(executor, empty_directory(), outcomes);

    bootstrap.viewer(author(AUTHOR_A));
    let failed = timeout(Duration::from_secs(5), reported.recv())
        .await
        .expect("the failure should be reported")
        .expect("the channel should stay open");
    assert!(matches!(
        failed,
        crate::retrieval_types::RetrievalOutcome::Completed { result: Err(_), .. }
    ));
    bootstrap.viewer(author(AUTHOR_A));

    requested_authors(&mut retrievals).await;
    assert_eq!(
        requested_authors(&mut retrievals).await,
        serde_json::json!([AUTHOR_A])
    );
}

#[tokio::test]
async fn ingested_relay_lists_reach_the_directory() {
    let (executor, _retrievals) = recording_executor();
    let (outcomes, _sink) = mpsc::unbounded_channel();
    let directory = empty_directory();
    let bootstrap = OutboxBootstrap::new(executor, directory.clone(), outcomes);
    let follow = Keys::generate();

    bootstrap.track_follows(vec![follow.public_key()]).await;
    bootstrap
        .ingest(&[relay_list_event(&follow, "wss://follow.write")])
        .await;

    let relays = directory.read().await.discovery_relays(12);
    assert_eq!(relays.last(), Some(&"wss://follow.write".to_owned()));
}
