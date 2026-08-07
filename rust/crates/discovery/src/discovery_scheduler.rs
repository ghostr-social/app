//! Event-driven relay retrieval queue with a bounded worker pool.

use crate::plan_executor::{PlanExecutor, PlanFailure, PlanPage};
use crate::retrieval_queue::{FeedContext, RetrievalQueue};
use crate::scheduler_feeds::FeedBook;
use crate::scheduler_hunt::HuntToken;
use crate::scheduler_queries::{QueryBook, QueryResult};
use crate::search_queries::QueryPlan;
use crate::video_filters::DiscoveryRequest;
use ghostr_engine::inventory_controller::Mode;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, Timestamp};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::mpsc::WeakUnboundedSender;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::AbortHandle;

mod handle;

/// Mirrors Dart's `maxConcurrentRequests` worker-pool cap.
pub fn max_concurrent_requests(level: DataUsageLevel) -> usize {
    match level {
        DataUsageLevel::Conservative => 2,
        DataUsageLevel::Balanced => 4,
        DataUsageLevel::Aggressive => 6,
    }
}

/// Control events the scheduler reacts to.
#[derive(Debug)]
pub(crate) enum DiscoveryCommand {
    /// Interactive feed/search/tag load; focuses its context.
    OpenFeed {
        context: FeedContext,
        request: DiscoveryRequest,
    },
    /// Interactive older-page load for an open feed; an explicit
    /// cursor (plan §2 `ffi_load_more`) wins over the tracked one.
    LoadMore {
        context: FeedContext,
        older_than: Option<Timestamp>,
    },
    /// Reorders queued work in the viewer's favor without loading.
    Focus(FeedContext),
    /// Background work (trending, backfill); never steals focus.
    Background {
        context: FeedContext,
        request: DiscoveryRequest,
    },
    /// Ends queued, running, and delayed work for a closed feed.
    CloseFeed(FeedContext),
    /// Rust-owned continuation of a continuous feed.
    ContinueFeed {
        context: FeedContext,
        head: bool,
        token: HuntToken,
    },
    /// Rust-owned retry of a canonical feed after relay failure.
    RetryFeed {
        context: FeedContext,
        token: HuntToken,
    },
    /// A generic read shares the queue but answers only its caller.
    Query {
        context: FeedContext,
        plan: QueryPlan,
        reply: oneshot::Sender<QueryResult>,
    },
    /// Live data-usage knob change.
    SetDataUsage(DataUsageLevel),
    /// Drops queued session work and every pending generic reply.
    ResetSession { reply: oneshot::Sender<()> },
}

/// One executed retrieval's role in feed assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalPurpose {
    Head,
    Older,
}

#[derive(Clone, Debug)]
pub enum RetrievalOutcome {
    Started {
        context: FeedContext,
    },
    Progress {
        context: FeedContext,
        event: Box<Event>,
    },
    Completed {
        context: FeedContext,
        result: Result<Vec<Event>, PlanFailure>,
        purpose: RetrievalPurpose,
    },
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
    /// Inventory mode transitions from the delivery manager
    /// (`start_delivery_manager_with_modes`), plan §5.4.
    pub modes: watch::Receiver<Mode>,
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
    pub(crate) modes: watch::Receiver<Mode>,
    pub(crate) modes_live: bool,
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
            modes: config.modes,
            modes_live: true,
        }
    }
}
