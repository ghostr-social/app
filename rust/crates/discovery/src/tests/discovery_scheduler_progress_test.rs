use super::scheduler_support::{context, next_outcome, note_at, request};
use crate::discovery_scheduler::{
    start_discovery_scheduler, DiscoverySchedulerConfig, RetrievalOutcome,
};
use crate::plan_executor::{EventProgress, PlanExecutor, PlanFuture, PlannedRetrieval};
use ghostr_engine::inventory_controller::Mode;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::Event;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Semaphore};

struct ProgressiveExecutor {
    event: Event,
    gate: Arc<Semaphore>,
}

impl PlanExecutor for ProgressiveExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        self.execute_with_progress(retrieval, mpsc::channel(1).0)
    }

    fn execute_with_progress(
        &self,
        _retrieval: PlannedRetrieval,
        progress: EventProgress,
    ) -> PlanFuture {
        let event = self.event.clone();
        let gate = self.gate.clone();
        Box::pin(async move {
            progress
                .send(event.clone())
                .await
                .expect("scheduler listens");
            gate.acquire().await.expect("test gate").forget();
            Ok(vec![event])
        })
    }
}

#[tokio::test]
async fn publishes_events_before_the_retrieval_settles() {
    let gate = Arc::new(Semaphore::new(0));
    let executor = Arc::new(ProgressiveExecutor {
        event: note_at(40),
        gate: gate.clone(),
    });
    let (sender, mut outcomes) = mpsc::unbounded_channel();
    let (_, modes) = watch::channel(Mode::Comfort);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor,
        level: DataUsageLevel::Conservative,
        modes,
        outcomes: sender,
    });

    handle.open_feed(context("search"), request());
    let progress = next_outcome(&mut outcomes).await;
    assert!(matches!(progress, RetrievalOutcome::Progress { .. }));

    gate.add_permits(1);
    assert!(matches!(
        next_outcome(&mut outcomes).await,
        RetrievalOutcome::Completed { .. }
    ));
}
