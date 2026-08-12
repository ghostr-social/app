use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::retrieval_types::PlanFailure;
use crate::scheduler::{start_discovery_scheduler, DiscoverySchedulerConfig};
use crate::tests::scheduler_support::{context, next_outcome, next_started, note_at, request};
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use nostr_sdk::Timestamp;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch, Semaphore};

struct BusyOlderExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    gate: Arc<Semaphore>,
    calls: AtomicUsize,
}

impl PlanExecutor for BusyOlderExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.starts.send(retrieval);
        let gate = self.gate.clone();
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            match call {
                0 => Err(PlanFailure::new("head offline")),
                1 => {
                    gate.acquire().await.expect("test gate").forget();
                    Ok(Vec::new())
                }
                _ => Ok(vec![note_at(40)]),
            }
        })
    }
}

#[tokio::test(start_paused = true)]
async fn a_retry_deferred_by_older_work_remains_pending() {
    let (starts, mut started) = mpsc::unbounded_channel();
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let (_, demand) = watch::channel(DiscoveryDemand::Hold);
    let gate = Arc::new(Semaphore::new(0));
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(BusyOlderExecutor {
            starts,
            gate: gate.clone(),
            calls: AtomicUsize::new(0),
        }),
        level: DataUsageLevel::Conservative,
        demand,
        outcomes,
    });
    let feed = context("main");
    handle.open_feed(feed.clone(), request());
    next_started(&mut started).await;
    next_outcome(&mut reported).await;
    handle.load_more(feed, Some(Timestamp::from(39)));
    next_started(&mut started).await;

    tokio::time::advance(Duration::from_millis(750)).await;
    tokio::task::yield_now().await;
    gate.add_permits(1);
    next_outcome(&mut reported).await;
    tokio::time::advance(Duration::from_millis(750)).await;
    tokio::task::yield_now().await;

    assert!(
        started.try_recv().is_ok(),
        "busy work must not consume a retry backoff rung"
    );
}
