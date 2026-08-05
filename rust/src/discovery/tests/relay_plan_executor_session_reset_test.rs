//! A generic query started before reset resolves as stale, not as data.

use crate::discovery::event_cache::client_with_event_cache;
use crate::discovery::event_queries::plan_event_queries;
use crate::discovery::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::discovery::relay_plan_executor::RelayPlanExecutor;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::discovery::session_generation::SessionGeneration;
use crate::discovery::tests::outbox_support::empty_directory;
use crate::engine::DataUsageLevel;
use nostr_sdk::{Filter, Kind};
use std::sync::Arc;

#[tokio::test]
async fn stale_query_is_rejected_before_it_can_return_events() {
    let executor = RelayPlanExecutor::new(
        Arc::new(client_with_event_cache()),
        Vec::new(),
        empty_directory(),
        DataUsageLevel::Balanced,
    );
    let stale = SessionGeneration::initial();
    executor.cache().reset_session(stale.next()).await;
    let retrieval = PlannedRetrieval {
        context: FeedContext::for_session("old-query", stale),
        priority: RetrievalPriority::Enrichment,
        plan: plan_event_queries(vec![Filter::new().kind(Kind::TextNote)]),
    };

    let failure = executor.execute(retrieval).await.expect_err("stale query");

    assert_eq!(failure.message, "the Nostr session was reset");
}
