//! Opening an author-scoped feed starts its relay-list chase immediately.

use crate::discovery::outbox_bootstrap::OutboxBootstrap;
use crate::discovery::tests::outbox_support::{empty_directory, recording_executor};
use crate::discovery::tests::support::{author, filter_json, AUTHOR_A};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn feed_authors_are_chased_for_relay_lists() {
    let (executor, mut retrievals) = recording_executor();
    let (outcomes, _receiver) = mpsc::unbounded_channel();
    let bootstrap = OutboxBootstrap::new(executor, empty_directory(), outcomes);

    bootstrap.authors(&[author(AUTHOR_A)]);
    let retrieval = timeout(Duration::from_secs(5), retrievals.recv())
        .await
        .expect("the chase starts")
        .expect("the recorder stays alive");

    assert_eq!(
        filter_json(&retrieval.plan.queries[0].filter)["authors"],
        serde_json::json!([AUTHOR_A]),
    );
}
