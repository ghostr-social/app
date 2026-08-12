//! Playable content permanently ends cold-start retry for one feed generation.

use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::retrieval_types::{EventProgress, PlanFailure, RetrievalOutcome};
use crate::scheduler::{start_discovery_scheduler, DiscoverySchedulerConfig};
use crate::tests::scheduler_support::{context, next_outcome, next_started, note_at, request};
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch, Semaphore};
use tokio::time::timeout;

struct WarmThenFail {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    calls: AtomicUsize,
    first_page_gate: Arc<Semaphore>,
}

impl PlanExecutor for WarmThenFail {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        self.execute_with_progress(retrieval, mpsc::channel(1).0)
    }

    fn execute_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        progress: EventProgress,
    ) -> PlanFuture {
        let _ = self.starts.send(retrieval);
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let first_page_gate = self.first_page_gate.clone();
        Box::pin(async move {
            if call == 0 {
                progress.send(note_at(40)).await.expect("progress");
                first_page_gate
                    .acquire()
                    .await
                    .expect("first page gate")
                    .forget();
            }
            Err(PlanFailure::new("relay offline"))
        })
    }
}

#[tokio::test(start_paused = true)]
async fn expansion_failure_does_not_restart_retry_after_playable_progress() {
    let (starts, mut started) = mpsc::unbounded_channel();
    let (outcome_sender, mut outcomes) = mpsc::unbounded_channel();
    let (demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
    let first_page_gate = Arc::new(Semaphore::new(0));
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(WarmThenFail {
            starts,
            calls: AtomicUsize::new(0),
            first_page_gate: first_page_gate.clone(),
        }),
        level: DataUsageLevel::Conservative,
        demand,
        outcomes: outcome_sender,
    });
    handle.open_feed(context("main"), request());
    next_started(&mut started).await;
    assert!(matches!(
        next_outcome(&mut outcomes).await,
        RetrievalOutcome::Progress { .. }
    ));
    first_page_gate.add_permits(1);
    next_outcome(&mut outcomes).await;

    demand_sender
        .send(DiscoveryDemand::Expand)
        .expect("scheduler subscribed");
    next_started(&mut started).await;
    next_outcome(&mut outcomes).await;
    next_outcome(&mut outcomes).await;
    tokio::time::advance(Duration::from_millis(750)).await;

    assert!(
        timeout(Duration::from_millis(50), started.recv())
            .await
            .is_err(),
        "a warm feed must not restart cold retry polling"
    );
}
