//! Scheduler reset promptly cancels an old account's generic query reply.

use super::scheduler_support::{context, next_outcome, no_start, request};
use super::scheduler_support::{next_started, start_scheduler};
use super::scripted_scheduler_support::scripted_scheduler;
use crate::discovery::event_queries::plan_event_queries;
use crate::discovery::scheduler_feeds::FEED_REFRESH_BACKOFF;
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

#[tokio::test(start_paused = true)]
async fn reset_cancels_a_delayed_query_hunt() {
    let mut harness = scripted_scheduler(vec![Vec::new()]);
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    harness.handle.open_feed(context("search"), query);
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    harness
        .handle
        .reset_session()
        .await
        .expect("scheduler reset");
    tokio::time::advance(FEED_REFRESH_BACKOFF).await;

    no_start(&mut harness.started).await;
}
