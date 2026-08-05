//! A failed relay read serves matching warm rows, but never hides a cold failure.

use super::event_cache_support::note;
use super::outbox_support::shared_directory;
use crate::discovery::event_cache::client_with_event_cache;
use crate::discovery::event_queries::plan_event_queries;
use crate::discovery::outbox_directory::OutboxDirectory;
use crate::discovery::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::discovery::relay_plan_executor::RelayPlanExecutor;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::engine::DataUsageLevel;
use nostr_sdk::{Client, Filter, Kind};
use std::sync::Arc;

fn executor(client: Arc<Client>) -> RelayPlanExecutor {
    let directory = OutboxDirectory::new(vec!["not-a-relay".to_owned()]);
    RelayPlanExecutor::new(
        client,
        Vec::new(),
        shared_directory(directory),
        DataUsageLevel::Balanced,
    )
}

fn retrieval() -> PlannedRetrieval {
    PlannedRetrieval {
        context: FeedContext::new("offline-query"),
        priority: RetrievalPriority::Enrichment,
        plan: plan_event_queries(vec![Filter::new().kind(Kind::TextNote)]),
    }
}

#[tokio::test]
async fn failed_network_query_returns_matching_warm_rows() {
    let client = Arc::new(client_with_event_cache());
    let executor = executor(client);
    executor.cache().remember(&[note(100)]).await;

    let events = executor.execute(retrieval()).await.expect("warm fallback");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].created_at.as_u64(), 100);
}

#[tokio::test]
async fn failed_network_query_still_errors_when_cache_is_empty() {
    let executor = executor(Arc::new(client_with_event_cache()));

    assert!(executor.execute(retrieval()).await.is_err());
}
