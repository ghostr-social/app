use crate::plan_executor::{PlanExecutor, PlanFuture, PlanPage, PlanPageFuture, PlannedRetrieval};
use crate::query::events::plan_event_queries;
use crate::scheduler::{start_discovery_scheduler, DiscoverySchedulerConfig};
use crate::session_generation::SessionGeneration;
use crate::tests::scheduler_support::note_at;
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use nostr_sdk::{Filter, Kind};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

struct IncompleteQueryExecutor;

impl PlanExecutor for IncompleteQueryExecutor {
    fn execute(&self, _: PlannedRetrieval) -> PlanFuture {
        unreachable!("scheduler uses page execution")
    }

    fn execute_page(&self, _: PlannedRetrieval) -> PlanPageFuture {
        Box::pin(async {
            Ok(PlanPage {
                events: vec![note_at(40)],
                cursor: None,
                complete: false,
                repost_retry: Default::default(),
            })
        })
    }
}

#[tokio::test]
async fn generic_query_rejects_an_incomplete_relay_page() {
    let (outcomes, _) = mpsc::unbounded_channel();
    let (_, demand) = watch::channel(DiscoveryDemand::Hold);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(IncompleteQueryExecutor),
        level: DataUsageLevel::Conservative,
        demand,
        outcomes,
    });
    let plan = plan_event_queries(vec![Filter::new().kind(Kind::Reaction)]);

    let result = handle.query(SessionGeneration::initial(), plan).await;

    assert!(
        result.is_err(),
        "partial events are not a complete query result"
    );
}
