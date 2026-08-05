//! Scheduler reset promptly cancels an old account's generic query reply.

use super::scheduler_support::{next_started, start_scheduler};
use crate::discovery::event_queries::plan_event_queries;
use crate::discovery::session_generation::SessionGeneration;
use crate::engine::DataUsageLevel;
use nostr_sdk::{Filter, Kind};

#[tokio::test]
async fn reset_cancels_an_inflight_generic_query_reply() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    let handle = harness.handle.clone();
    let session = SessionGeneration::initial();
    let plan = plan_event_queries(vec![Filter::new().kind(Kind::Reaction)]);
    let query = tokio::spawn(async move { handle.query(session, plan).await });
    next_started(&mut harness.started).await;

    harness
        .handle
        .reset_session()
        .await
        .expect("scheduler reset");

    let failure = query
        .await
        .expect("query task")
        .expect_err("old query must be cancelled");
    assert_eq!(failure.message, "the discovery query was cancelled");
    harness.gate.add_permits(1);
}
