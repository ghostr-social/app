//! Event-driven relay retrieval queue with a bounded worker pool.

pub(crate) mod commands;
pub mod control;
pub(crate) mod deferred_reposts;
pub(crate) mod event_loop;
pub(crate) mod feeds;
pub mod hunt;
pub(crate) mod plans;
pub(crate) mod progress;
pub mod queries;
pub mod queue;
pub(crate) mod retry;
pub(crate) mod session;

use crate::plan_executor::{PlanExecutor, PlanPage};
use crate::query::search::QueryPlan;
use crate::retrieval_types::{FeedContext, PlanFailure, RetrievalOutcome, RetrievalPurpose};
use crate::scheduler::feeds::FeedBook;
use crate::scheduler::hunt::HuntToken;
use crate::scheduler::queries::QueryBook;
use crate::scheduler::queue::RetrievalQueue;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::DataUsageLevel;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::mpsc::WeakUnboundedSender;
use tokio::sync::{mpsc, watch};
use tokio::task::AbortHandle;

mod handle;

pub(crate) use commands::{ControlCommand, DiscoveryCommand, FeedCommand, WorkCommand};

/// Mirrors Dart's `maxConcurrentRequests` worker-pool cap.
pub(crate) fn max_concurrent_requests(level: DataUsageLevel) -> usize {
    match level {
        DataUsageLevel::Conservative => 2,
        DataUsageLevel::Balanced => 4,
        DataUsageLevel::Aggressive => 6,
    }
}

/// Cloneable control handle; sends never block. The scheduler task
/// ends once every handle clone is dropped.
#[derive(Clone, Debug)]
pub struct DiscoveryHandle {
    sender: mpsc::UnboundedSender<DiscoveryCommand>,
    query_sequence: Arc<AtomicU64>,
}

/// Everything the scheduler owns or reaches, as one typed object.
pub struct DiscoverySchedulerConfig {
    pub executor: Arc<dyn PlanExecutor>,
    pub level: DataUsageLevel,
    /// Resource-driven candidate demand from the delivery manager.
    pub demand: watch::Receiver<DiscoveryDemand>,
    pub outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
}

/// Starts the scheduler task and returns its control handle.
pub fn start_discovery_scheduler(config: DiscoverySchedulerConfig) -> DiscoveryHandle {
    let (sender, commands) = mpsc::unbounded_channel();
    tokio::spawn(run(SchedulerWorker::create(
        config,
        commands,
        sender.downgrade(),
    )));
    DiscoveryHandle {
        sender,
        query_sequence: Arc::new(AtomicU64::new(0)),
    }
}

async fn run(mut worker: SchedulerWorker) {
    while worker.step().await {}
}

pub(crate) struct FinishedRetrieval {
    pub(crate) task_id: u64,
    pub(crate) context: FeedContext,
    pub(crate) result: Result<PlanPage, PlanFailure>,
    pub(crate) purpose: RetrievalPurpose,
    pub(crate) had_playable_progress: bool,
}

pub(crate) struct ActiveRetrieval {
    pub(crate) context: FeedContext,
    pub(crate) abort: AbortHandle,
}

pub(crate) struct SchedulerWorker {
    pub(crate) queue: RetrievalQueue<QueryPlan>,
    pub(crate) feeds: FeedBook,
    pub(crate) deferred_reposts: deferred_reposts::DeferredRepostBook,
    pub(crate) queries: QueryBook,
    pub(crate) executor: Arc<dyn PlanExecutor>,
    pub(crate) max_concurrent: usize,
    pub(crate) outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
    pub(crate) finished_sender: mpsc::UnboundedSender<FinishedRetrieval>,
    pub(crate) finished: mpsc::UnboundedReceiver<FinishedRetrieval>,
    pub(crate) tasks: HashMap<u64, ActiveRetrieval>,
    pub(crate) hunts: HashMap<FeedContext, AbortHandle>,
    pub(crate) retry_attempts: HashMap<FeedContext, usize>,
    pub(crate) pending_feed_retries: HashMap<FeedContext, HuntToken>,
    pub(crate) pending_feed_hunts: HashMap<FeedContext, HuntToken>,
    pub(crate) next_hunt_token: u64,
    pub(crate) next_task_id: u64,
    pub(crate) commands: mpsc::UnboundedReceiver<DiscoveryCommand>,
    pub(crate) command_sender: WeakUnboundedSender<DiscoveryCommand>,
    pub(crate) demand: watch::Receiver<DiscoveryDemand>,
    pub(crate) demand_live: bool,
}

impl SchedulerWorker {
    fn create(
        config: DiscoverySchedulerConfig,
        commands: mpsc::UnboundedReceiver<DiscoveryCommand>,
        command_sender: WeakUnboundedSender<DiscoveryCommand>,
    ) -> Self {
        let (finished_sender, finished) = mpsc::unbounded_channel();
        Self {
            queue: RetrievalQueue::new(),
            feeds: FeedBook::default(),
            deferred_reposts: deferred_reposts::DeferredRepostBook::default(),
            queries: QueryBook::default(),
            executor: config.executor,
            max_concurrent: max_concurrent_requests(config.level),
            outcomes: config.outcomes,
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
            command_sender,
            demand: config.demand,
            demand_live: true,
        }
    }
}
