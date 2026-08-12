use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::retrieval_types::{EventProgress, PlanFailure, RetrievalOutcome};
use crate::scheduler::{start_discovery_scheduler, DiscoverySchedulerConfig};
use crate::tests::scheduler_support::{
    context, next_outcome, next_started, no_start, note_at, request,
};
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch, Semaphore};

struct ProgressThenFailure {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    gate: Arc<Semaphore>,
}

impl PlanExecutor for ProgressThenFailure {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        self.execute_with_progress(retrieval, mpsc::channel(1).0)
    }

    fn execute_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        progress: EventProgress,
    ) -> PlanFuture {
        let _ = self.starts.send(retrieval);
        let gate = self.gate.clone();
        Box::pin(async move {
            progress.send(note_at(40)).await.expect("scheduler listens");
            gate.acquire().await.expect("test gate").forget();
            Err(PlanFailure::new("additive relay failed"))
        })
    }
}

#[tokio::test(start_paused = true)]
async fn playable_progress_prevents_a_final_error_retry_storm() {
    let (starts, mut started) = mpsc::unbounded_channel();
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let (_, demand) = watch::channel(DiscoveryDemand::Hold);
    let gate = Arc::new(Semaphore::new(0));
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(ProgressThenFailure {
            starts,
            gate: gate.clone(),
        }),
        level: DataUsageLevel::Conservative,
        demand,
        outcomes,
    });
    handle.open_feed(context("main"), request());
    next_started(&mut started).await;
    assert!(matches!(
        next_outcome(&mut reported).await,
        RetrievalOutcome::Progress { .. }
    ));
    gate.add_permits(1);
    assert!(matches!(
        next_outcome(&mut reported).await,
        RetrievalOutcome::Completed { result: Err(_), .. }
    ));

    tokio::time::advance(Duration::from_millis(750)).await;
    no_start(&mut started).await;
}
