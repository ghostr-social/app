use crate::plan_executor::{PlanExecutor, PlanFuture, PlanPage, PlanPageFuture, PlannedRetrieval};
use crate::retrieval_types::{EventProgress, RetrievalOutcome};
use crate::scheduler::{start_discovery_scheduler, DiscoverySchedulerConfig};
use crate::tests::scheduler_support::{context, next_outcome, next_started, note_at, request};
use core::time::Duration;
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

struct IncompleteExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
}

impl PlanExecutor for IncompleteExecutor {
    fn execute(&self, _: PlannedRetrieval) -> PlanFuture {
        unreachable!("scheduler uses page execution")
    }

    fn execute_page_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        _: EventProgress,
    ) -> PlanPageFuture {
        let _ = self.starts.send(retrieval);
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

#[tokio::test(start_paused = true)]
async fn playable_incomplete_head_retries_without_committing_a_cursor() {
    let (starts, mut started) = mpsc::unbounded_channel();
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let (_, demand) = watch::channel(DiscoveryDemand::Hold);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(IncompleteExecutor { starts }),
        level: DataUsageLevel::Conservative,
        demand,
        outcomes,
    });
    handle.open_feed(context("main"), request());
    next_started(&mut started).await;

    assert!(matches!(
        next_outcome(&mut reported).await,
        RetrievalOutcome::Completed {
            result: Ok(events), cursor: None, complete: false, ..
        } if events.len() == 1
    ));
    tokio::time::advance(Duration::from_millis(750)).await;
    next_started(&mut started).await;
}
