//! Event-driven discovery scheduler: the one queue every relay
//! retrieval flows through, with a bounded worker pool as the
//! data-usage knob. Parity source: RetrievalScheduler in
//! lib/core/work/retrieval_scheduler.dart, wired by
//! lib/app/production_video_delivery.dart. Commands arrive over a
//! channel and executed batches stream out — no polling loops.

use crate::discovery::plan_executor::{PlanExecutor, PlanFailure};
use crate::discovery::retrieval_queue::{FeedContext, RetrievalQueue};
use crate::discovery::scheduler_feeds::FeedBook;
use crate::discovery::search_queries::QueryPlan;
use crate::discovery::video_filters::DiscoveryRequest;
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;
use nostr_sdk::{Event, Timestamp};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

/// Worker-pool cap per data-usage level; mirrors
/// `maxConcurrentRequests` (2/4/6) in
/// lib/features/settings/domain/data_usage_level.dart.
pub fn max_concurrent_requests(level: DataUsageLevel) -> usize {
    match level {
        DataUsageLevel::Conservative => 2,
        DataUsageLevel::Balanced => 4,
        DataUsageLevel::Aggressive => 6,
    }
}

/// Control events the scheduler reacts to.
#[derive(Clone, Debug)]
pub enum DiscoveryCommand {
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
    /// Live data-usage knob change.
    SetDataUsage(DataUsageLevel),
}

/// One executed retrieval, streamed to feed assembly.
#[derive(Clone, Debug)]
pub struct RetrievalOutcome {
    pub context: FeedContext,
    pub result: Result<Vec<Event>, PlanFailure>,
}

/// Cloneable control handle; sends never block. The scheduler task
/// ends once every handle clone is dropped.
#[derive(Clone, Debug)]
pub struct DiscoveryHandle {
    sender: mpsc::UnboundedSender<DiscoveryCommand>,
}

impl DiscoveryHandle {
    pub fn open_feed(&self, context: FeedContext, request: DiscoveryRequest) {
        let _ = self.sender.send(DiscoveryCommand::OpenFeed { context, request });
    }

    pub fn load_more(&self, context: FeedContext, older_than: Option<Timestamp>) {
        let _ = self.sender.send(DiscoveryCommand::LoadMore { context, older_than });
    }

    pub fn focus(&self, context: FeedContext) {
        let _ = self.sender.send(DiscoveryCommand::Focus(context));
    }

    pub fn background(&self, context: FeedContext, request: DiscoveryRequest) {
        let _ = self.sender.send(DiscoveryCommand::Background { context, request });
    }

    pub fn set_data_usage(&self, level: DataUsageLevel) {
        let _ = self.sender.send(DiscoveryCommand::SetDataUsage(level));
    }
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
    tokio::spawn(run(SchedulerWorker::create(config, commands)));
    DiscoveryHandle { sender }
}

async fn run(mut worker: SchedulerWorker) {
    while worker.step().await {}
}

pub(crate) struct FinishedRetrieval {
    pub(crate) context: FeedContext,
    pub(crate) result: Result<Vec<Event>, PlanFailure>,
}

pub(crate) struct SchedulerWorker {
    pub(crate) queue: RetrievalQueue<QueryPlan>,
    pub(crate) feeds: FeedBook,
    pub(crate) executor: Arc<dyn PlanExecutor>,
    pub(crate) max_concurrent: usize,
    pub(crate) outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
    pub(crate) finished_sender: mpsc::UnboundedSender<FinishedRetrieval>,
    pub(crate) finished: mpsc::UnboundedReceiver<FinishedRetrieval>,
    pub(crate) commands: mpsc::UnboundedReceiver<DiscoveryCommand>,
    pub(crate) modes: watch::Receiver<Mode>,
    pub(crate) modes_live: bool,
}

impl SchedulerWorker {
    fn create(
        config: DiscoverySchedulerConfig,
        commands: mpsc::UnboundedReceiver<DiscoveryCommand>,
    ) -> Self {
        let (finished_sender, finished) = mpsc::unbounded_channel();
        Self {
            queue: RetrievalQueue::new(),
            feeds: FeedBook::default(),
            executor: config.executor,
            max_concurrent: max_concurrent_requests(config.level),
            outcomes: config.outcomes,
            finished_sender,
            finished,
            commands,
            modes: config.modes,
            modes_live: true,
        }
    }
}
