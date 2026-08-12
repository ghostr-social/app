//! A queued query continuation cannot steal a newer hunt lifecycle.

use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::scheduler::feeds::{FeedBook, FEED_REFRESH_BACKOFF};
use crate::scheduler::queries::QueryBook;
use crate::scheduler::queue::RetrievalQueue;
use crate::scheduler::SchedulerWorker;
use crate::tests::scheduler_support::{context, request};
use ghostr_engine::adaptive::DiscoveryDemand;
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
async fn stale_query_continuation_preserves_the_newer_hunt() {
    let (outcome_sender, _outcomes) = mpsc::unbounded_channel();
    let (finished_sender, finished) = mpsc::unbounded_channel();
    let (command_sender, commands) = mpsc::unbounded_channel();
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
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
        next_task_id: 0,
        commands,
        command_sender: command_sender.downgrade(),
        demand,
        demand_live: true,
    };
    let feed = context("search");
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    worker.feeds.open(feed.clone(), query);
    worker.advance_feed_hunt(feed.clone());
    tokio::task::yield_now().await;
    tokio::time::advance(FEED_REFRESH_BACKOFF).await;
    tokio::task::yield_now().await;
    let stale = worker.commands.try_recv().expect("old hunt command");

    worker.advance_feed_hunt(feed.clone());
    assert!(worker.hunts.contains_key(&feed), "new hunt precondition");
    worker.apply_command(stale);

    assert!(
        worker.hunts.contains_key(&feed) && !worker.queue.has_pending(&feed),
        "the stale command consumed the newer hunt or queued a duplicate refresh"
    );
}
