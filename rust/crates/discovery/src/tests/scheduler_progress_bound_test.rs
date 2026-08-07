use super::scheduler_support::{context, next_outcome, note_at, request};
use crate::discovery_scheduler::{
    start_discovery_scheduler, DiscoverySchedulerConfig, RetrievalOutcome,
};
use crate::plan_executor::{EventProgress, PlanExecutor, PlanFuture, PlannedRetrieval};
use ghostr_engine::{inventory_controller::Mode, DataUsageLevel};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

struct BurstExecutor;
const BURST_SIZE: usize = 256;

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
            for _ in 0..BURST_SIZE {
                progress.send(event.clone()).await.expect("live progress");
            }
            Ok(vec![event])
        })
    }
}

#[tokio::test]
async fn relay_bursts_stream_every_event_through_bounded_backpressure() {
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

    assert_eq!(progress, BURST_SIZE);
}
