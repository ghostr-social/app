//! Generic reads use scheduler enrichment slots and return only to their caller.

use crate::query::events::plan_event_queries;
use crate::retrieval_types::{RetrievalOutcome, RetrievalPriority};
use crate::session_generation::SessionGeneration;
use crate::tests::scheduler_support::{next_started, start_scheduler};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Filter, Kind};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test(start_paused = true)]
async fn generic_query_returns_without_entering_feed_outcomes() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    let plan = plan_event_queries(vec![Filter::new().kind(Kind::Reaction)]);
    let handle = harness.handle.clone();
    let session = SessionGeneration::initial().next();
    let result = tokio::spawn(async move { handle.query(session, plan).await });

    let started = next_started(&mut harness.started).await;
    assert_eq!(started.priority, RetrievalPriority::Enrichment);
    assert_eq!(started.context.session(), session);
    harness.gate.add_permits(1);

    assert!(result.await.expect("query task").expect("query").is_empty());
    no_outcome(&mut harness.outcomes).await;
}

async fn no_outcome(outcomes: &mut mpsc::UnboundedReceiver<RetrievalOutcome>) {
    let result = timeout(Duration::from_millis(50), outcomes.recv()).await;
    assert!(
        result.is_err(),
        "generic queries must not enter feed outcomes"
    );
}
