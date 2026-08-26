//! Event-driven relay retrieval queue with a bounded worker pool.

pub(crate) mod commands;
pub mod control;
pub(crate) mod deferred_reposts;
pub(super) mod event_loop;
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
use core::sync::atomic::AtomicU64;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::DataUsageLevel;
use std::collections::{BTreeMap, HashMap};
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
    pub(super) task_id: u64,
    pub(super) context: FeedContext,
    pub(super) result: Result<PlanPage, PlanFailure>,
    pub(super) purpose: RetrievalPurpose,
    pub(super) had_playable_progress: bool,
}

pub(crate) struct ActiveRetrieval {
    pub(super) context: FeedContext,
    pub(super) abort: AbortHandle,
}

pub(crate) struct SchedulerWorker {
    pub(super) queue: RetrievalQueue<QueryPlan>,
    pub(super) feeds: FeedBook,
    pub(super) deferred_reposts: deferred_reposts::DeferredRepostBook,
    pub(super) queries: QueryBook,
    pub(super) executor: Arc<dyn PlanExecutor>,
    pub(super) max_concurrent: usize,
    pub(super) outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
    pub(super) finished_sender: mpsc::UnboundedSender<FinishedRetrieval>,
    pub(super) finished: mpsc::UnboundedReceiver<FinishedRetrieval>,
    pub(super) tasks: BTreeMap<u64, ActiveRetrieval>,
    pub(super) hunts: BTreeMap<FeedContext, AbortHandle>,
    pub(super) retry_attempts: HashMap<FeedContext, usize>,
    pub(super) pending_feed_retries: HashMap<FeedContext, HuntToken>,
    pub(super) pending_feed_hunts: HashMap<FeedContext, HuntToken>,
    pub(super) next_hunt_token: u64,
    pub(super) next_task_id: u64,
    pub(super) commands: mpsc::UnboundedReceiver<DiscoveryCommand>,
    pub(super) command_sender: WeakUnboundedSender<DiscoveryCommand>,
    pub(super) demand: watch::Receiver<DiscoveryDemand>,
    pub(super) demand_live: bool,
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
            tasks: BTreeMap::new(),
            hunts: BTreeMap::new(),
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
