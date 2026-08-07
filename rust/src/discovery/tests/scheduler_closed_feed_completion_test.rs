//! A completion already queued at close time belongs to the closed lifecycle.

use super::scheduler_support::{context, request};
use crate::discovery::discovery_scheduler::{
    ActiveRetrieval, DiscoveryCommand, FinishedRetrieval, RetrievalPurpose, SchedulerWorker,
};
use crate::discovery::plan_executor::{PlanExecutor, PlanFailure, PlanFuture, PlannedRetrieval};
use crate::discovery::retrieval_queue::RetrievalQueue;
use crate::discovery::scheduler_feeds::FeedBook;
use crate::discovery::scheduler_queries::QueryBook;
use crate::engine::inventory_controller::Mode;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

struct NeverExecutor;

impl PlanExecutor for NeverExecutor {
    fn execute(&self, _retrieval: PlannedRetrieval) -> PlanFuture {
        Box::pin(std::future::pending())
    }
}

#[tokio::test(start_paused = true)]
async fn queued_completion_is_ignored_after_its_feed_closes() {
    let (outcome_sender, mut outcomes) = mpsc::unbounded_channel();
    let (finished_sender, finished) = mpsc::unbounded_channel();
    let (command_sender, commands) = mpsc::unbounded_channel();
    let (_mode_sender, modes) = watch::channel(Mode::Comfort);
    let mut worker = SchedulerWorker {
        queue: RetrievalQueue::new(),
        feeds: FeedBook::default(),
        queries: QueryBook::default(),
        executor: Arc::new(NeverExecutor),
        max_concurrent: 2,
        outcomes: outcome_sender,
        finished_sender,
        finished,
        tasks: HashMap::new(),
        hunts: HashMap::new(),
        retry_attempts: HashMap::new(),
        pending_feed_retries: HashMap::new(),
        pending_feed_hunts: HashMap::new(),
        next_hunt_token: 0,
        next_task_id: 1,
        commands,
        command_sender: command_sender.downgrade(),
        modes,
        modes_live: true,
    };
    let feed = context("main");
    worker.apply_command(DiscoveryCommand::OpenFeed {
        context: feed.clone(),
        request: request(),
    });
    let active = tokio::spawn(std::future::pending::<()>());
    worker.tasks.insert(
        0,
        ActiveRetrieval {
            context: feed.clone(),
            abort: active.abort_handle(),
        },
    );
    worker
        .finished_sender
        .send(FinishedRetrieval {
            task_id: 0,
            context: feed.clone(),
            result: Err(PlanFailure::new("offline")),
            purpose: RetrievalPurpose::Head,
            had_playable_progress: false,
        })
        .expect("queue completion");

    worker.apply_command(DiscoveryCommand::CloseFeed(feed.clone()));
    assert!(worker.step().await);

    let stale_outcome = outcomes.try_recv().is_ok();
    let retry_rearmed = worker.pending_feed_retries.contains_key(&feed);
    assert!(
        !stale_outcome && !retry_rearmed,
        "closed completion emitted={stale_outcome}, retry rearmed={retry_rearmed}"
    );
}
