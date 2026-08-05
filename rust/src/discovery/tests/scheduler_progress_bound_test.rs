use super::scheduler_support::{context, next_outcome, note_at, request};
use crate::discovery::discovery_scheduler::{
    start_discovery_scheduler, DiscoverySchedulerConfig, RetrievalOutcome,
};
use crate::discovery::plan_executor::{EventProgress, PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::discovery::scheduler_progress::MAX_PROGRESS_OUTCOMES;
use crate::engine::{inventory_controller::Mode, DataUsageLevel};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

struct BurstExecutor;

impl PlanExecutor for BurstExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let (progress, _) = mpsc::channel(1);
        self.execute_with_progress(retrieval, progress)
    }

    fn execute_with_progress(
        &self,
        _retrieval: PlannedRetrieval,
        progress: EventProgress,
    ) -> PlanFuture {
        Box::pin(async move {
            let event = note_at(40);
            for _ in 0..(MAX_PROGRESS_OUTCOMES * 4) {
                progress.send(event.clone()).await.expect("live progress");
            }
            Ok(vec![event])
        })
    }
}

#[tokio::test]
async fn relay_bursts_have_a_bounded_number_of_progress_outcomes() {
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let (_, modes) = watch::channel(Mode::Comfort);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(BurstExecutor),
        level: DataUsageLevel::Conservative,
        modes,
        outcomes,
    });
    handle.open_feed(context("bounded"), request());

    let mut progress = 0;
    loop {
        match next_outcome(&mut reported).await {
            RetrievalOutcome::Progress { .. } => progress += 1,
            RetrievalOutcome::Completed { .. } => break,
            RetrievalOutcome::Started { .. } => {}
        }
    }

    assert_eq!(progress, MAX_PROGRESS_OUTCOMES);
}
