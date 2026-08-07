//! Reopening one feed context supersedes its older retrieval generation.

use super::scheduler_support::{context, next_outcome, next_started, no_start, note_at, request};
use crate::discovery_scheduler::{
    start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig, RetrievalOutcome,
};
use crate::plan_executor::{PlanExecutor, PlanFailure, PlanFuture, PlannedRetrieval};
use ghostr_engine::{inventory_controller::Mode, DataUsageLevel};
use nostr_sdk::Event;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::timeout;

type Completion = oneshot::Receiver<Result<Vec<Event>, PlanFailure>>;

struct ControlledExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    completions: Mutex<VecDeque<Completion>>,
}

struct ControlledScheduler {
    handle: DiscoveryHandle,
    started: mpsc::UnboundedReceiver<PlannedRetrieval>,
    outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
}

impl PlanExecutor for ControlledExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.starts.send(retrieval);
        let completion = self.completions.lock().expect("completions").pop_front();
        Box::pin(async move {
            completion
                .expect("scripted completion")
                .await
                .expect("completion sender")
        })
    }
}

fn controlled_scheduler(
    completions: [oneshot::Receiver<Result<Vec<Event>, PlanFailure>>; 2],
) -> ControlledScheduler {
    let (starts, started) = mpsc::unbounded_channel();
    let executor = Arc::new(ControlledExecutor {
        starts,
        completions: Mutex::new(completions.into()),
    });
    let (outcome_sender, outcomes) = mpsc::unbounded_channel();
    let (_, modes) = watch::channel(Mode::Comfort);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor,
        level: DataUsageLevel::Conservative,
        modes,
        outcomes: outcome_sender,
    });
    ControlledScheduler {
        handle,
        started,
        outcomes,
    }
}

#[tokio::test(start_paused = true)]
async fn reopening_same_context_ignores_older_failure() {
    let (stale_sender, stale) = oneshot::channel();
    let (fresh_sender, fresh) = oneshot::channel();
    let mut harness = controlled_scheduler([stale, fresh]);
    let feed = context("main");

    harness.handle.open_feed(feed.clone(), request());
    next_started(&mut harness.started).await;
    harness.handle.open_feed(feed, request());
    next_started(&mut harness.started).await;

    fresh_sender.send(Ok(vec![note_at(40)])).expect("fresh");
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Ok(_), .. }
    ));
    let _ = stale_sender.send(Err(PlanFailure::new("stale offline")));

    assert!(
        timeout(Duration::from_millis(50), harness.outcomes.recv())
            .await
            .is_err(),
        "the superseded failure must not produce an outcome"
    );
    tokio::time::advance(Duration::from_millis(750)).await;
    no_start(&mut harness.started).await;
}
