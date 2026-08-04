//! Event-driven delivery manager (plan Phase 1 step 7): replaces the
//! old one-second poll loop. Focus updates, config changes, gateway
//! demand, and transfer completions arrive over channels; every event
//! triggers one replanning pass — there is no periodic wake-up.

use crate::engine::inventory_controller::Mode;
use crate::engine::{DataUsageLevel, EngineParams};
use crate::video::delivery_events::{
    command_channel, CommandReceiver, DeliveryCommand, DeliveryHandle,
};
use crate::video::delivery_inflight::InFlightChunks;
use crate::video::delivery_probes::ProbeBook;
use crate::video::delivery_retry::{RetryBook, RetryPolicy};
use crate::video::delivery_state::DeliveryState;
use crate::video::delivery_stats::StatsKeeper;
use crate::video::delivery_transfers::{InternalEvent, TransferContext};
use crate::video::outbound_media_client::MediaHttpClient;
use crate::video::partial_range_store::PartialRangeStore;
use crate::video::playback_demand::{DemandReceiver, DemandSignal};
use crate::video::progressive_posts::ServablePosts;
use crate::video::transfer_timeouts::TransferTimeouts;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Operational knobs outside the engine's tuning table.
#[derive(Clone, Copy, Debug)]
pub struct DeliveryTuning {
    /// Concurrent HEAD probes for unknown-size posts.
    pub probe_concurrency: usize,
    /// Backoff ladder and give-up budgets for failing sources.
    pub retry: RetryPolicy,
    /// Quiet period before persisting the host-stats snapshot.
    pub stats_debounce: Duration,
}

impl Default for DeliveryTuning {
    fn default() -> Self {
        Self {
            probe_concurrency: 2,
            retry: RetryPolicy::default(),
            stats_debounce: Duration::from_secs(2),
        }
    }
}

/// Everything the manager owns or reaches, as one typed object.
pub struct DeliveryManagerConfig {
    pub store: Arc<PartialRangeStore>,
    pub client: MediaHttpClient,
    pub posts: ServablePosts,
    pub stats_path: PathBuf,
    pub params: EngineParams,
    pub level: DataUsageLevel,
    pub tuning: DeliveryTuning,
}

/// Starts the manager task and returns its control handle. The task
/// drains, saves its stats, and ends once every handle is dropped.
pub fn start_delivery_manager(
    config: DeliveryManagerConfig,
    demand: DemandReceiver,
) -> DeliveryHandle {
    let (handle, commands) = command_channel();
    tokio::spawn(run(config, commands, demand, None));
    handle
}

/// Like [`start_delivery_manager`], but additionally exposes the
/// inventory mode transitions the discovery control loop subscribes
/// to (plan §5.4). The receiver starts at [`Mode::Hunger`], matching
/// a fresh controller.
pub fn start_delivery_manager_with_modes(
    config: DeliveryManagerConfig,
    demand: DemandReceiver,
) -> (DeliveryHandle, watch::Receiver<Mode>) {
    let (modes, mode_updates) = watch::channel(Mode::Hunger);
    let (handle, commands) = command_channel();
    tokio::spawn(run(config, commands, demand, Some(modes)));
    (handle, mode_updates)
}

async fn run(
    config: DeliveryManagerConfig,
    commands: CommandReceiver,
    demand: DemandReceiver,
    modes: Option<watch::Sender<Mode>>,
) {
    let mut worker = DeliveryWorker::create(config, commands, demand).await;
    if let Some(sender) = modes {
        worker.state.publish_modes(sender);
    }
    while worker.step().await {}
    worker.keeper.save_now().await;
}

pub(crate) struct DeliveryWorker {
    pub(crate) state: DeliveryState,
    pub(crate) keeper: StatsKeeper,
    pub(crate) inflight: InFlightChunks,
    pub(crate) probes: ProbeBook,
    pub(crate) retry: RetryBook,
    pub(crate) pending_demand: Option<DemandSignal>,
    pub(crate) ctx: TransferContext,
    pub(crate) posts: ServablePosts,
    commands: CommandReceiver,
    demand: DemandReceiver,
    events: mpsc::UnboundedReceiver<InternalEvent>,
}

enum Wake {
    Command(DeliveryCommand),
    Demand(DemandSignal),
    Internal(InternalEvent),
}

impl DeliveryWorker {
    async fn create(
        config: DeliveryManagerConfig,
        commands: CommandReceiver,
        demand: DemandReceiver,
    ) -> Self {
        let (events_sender, events) = mpsc::unbounded_channel();
        Self {
            state: DeliveryState::new(config.params, config.level),
            keeper: StatsKeeper::load(config.stats_path, config.tuning.stats_debounce).await,
            inflight: InFlightChunks::new(),
            probes: ProbeBook::new(config.tuning.probe_concurrency),
            retry: RetryBook::new(config.tuning.retry),
            pending_demand: None,
            ctx: TransferContext {
                client: config.client,
                store: config.store,
                events: events_sender,
                timeouts: TransferTimeouts::default(),
            },
            posts: config.posts,
            commands,
            demand,
            events,
        }
    }

    /// Waits for one event, applies it, and replans. Returns `false`
    /// only when the control channel is gone (shutdown).
    async fn step(&mut self) -> bool {
        let Some(wake) = self.next_wake().await else {
            return false;
        };
        self.apply(wake).await;
        self.reconcile().await;
        true
    }

    async fn next_wake(&mut self) -> Option<Wake> {
        tokio::select! {
            command = self.commands.recv() => command.map(Wake::Command),
            Some(signal) = self.demand.recv() => Some(Wake::Demand(signal)),
            Some(event) = self.events.recv() => Some(Wake::Internal(event)),
        }
    }

    async fn apply(&mut self, wake: Wake) {
        match wake {
            Wake::Command(DeliveryCommand::Focus(update)) => {
                self.state.apply_focus(update);
                self.refresh_servable();
            }
            Wake::Command(DeliveryCommand::Config(level)) => self.state.apply_level(level),
            Wake::Demand(signal) => self.pending_demand = Some(signal),
            Wake::Internal(event) => self.apply_internal(event).await,
        }
    }

    async fn apply_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::ChunkDone(done) => self.finish_chunk(done).await,
            InternalEvent::ProbeDone(done) => self.finish_probe(done).await,
            InternalEvent::CooldownOver(post) => self.retry.warm_up(&post),
            InternalEvent::SaveStats => self.keeper.save_now().await,
        }
    }
}
