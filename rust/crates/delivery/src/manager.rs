//! Event-driven delivery manager (plan Phase 1 step 7): replaces the
//! old one-second poll loop. Focus updates, config changes, gateway
//! demand, and transfer completions arrive over channels; every event
//! triggers one replanning pass — there is no periodic wake-up.
//!
//! The parts of that pass live beside this file: `reconcile` decides
//! what should be in flight, `transfers` and `workers` run it,
//! `completion` and `probe_completion` absorb the results, and
//! `retry`/`failure`/`pressure` decide what a failure means.

pub mod cache;
pub mod completion;
pub mod failure;
pub mod inflight;
pub mod plan;
pub mod pressure;
pub mod probe_completion;
pub mod reconcile;
pub mod reset;
pub mod retry;
pub mod state;
pub mod stats;
pub mod transfers;
pub mod wake;
pub mod workers;

use ghostr_engine::inventory_controller::Mode;
use ghostr_engine::{DataUsageLevel, EngineParams};
use crate::cache_registry::CacheRegistry;
use crate::debug::network::NetworkThrottle;
use crate::delivery_events::{command_channel, CommandReceiver, DeliveryHandle};
use crate::manager::pressure::StorePressure;
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::manager::stats::StatsKeeper;
use crate::manager::transfers::{InternalEvent, TransferContext};
use crate::manager::workers::DownloadWorkers;
use crate::probe::pool::MetadataProbePool;
use crate::mutable_priority_queue::MutablePriorityQueue;
use ghostr_net::outbound_media_client::MediaHttpClient;
use ghostr_partial_store::partial_range_store::capacity::DEFAULT_RECHECK;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use crate::playback_demand::{DemandReceiver, DemandSignal};
use ghostr_net::transfer_timeouts::TransferTimeouts;
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
    /// How long a post waits after the store refused its bytes. One
    /// capacity measurement is the earliest a new answer can exist.
    pub store_pressure_pause: Duration,
}

impl Default for DeliveryTuning {
    fn default() -> Self {
        Self {
            probe_concurrency: 2,
            retry: RetryPolicy::default(),
            stats_debounce: Duration::from_secs(2),
            store_pressure_pause: DEFAULT_RECHECK,
        }
    }
}

/// Everything the manager owns or reaches, as one typed object.
pub struct DeliveryManagerConfig {
    pub store: Arc<PartialRangeStore>,
    pub client: MediaHttpClient,
    pub cache: CacheRegistry,
    pub network: NetworkThrottle,
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
    pub(crate) downloads: DownloadWorkers,
    pub(crate) queue: MutablePriorityQueue,
    pub(crate) probes: MetadataProbePool,
    pub(crate) retry: RetryBook,
    pub(crate) pressure: StorePressure,
    pub(crate) pending_demand: Option<DemandSignal>,
    pub(crate) ctx: TransferContext,
    pub(crate) cache: CacheRegistry,
    pub(crate) commands: CommandReceiver,
    pub(crate) demand: DemandReceiver,
    pub(crate) events: mpsc::UnboundedReceiver<InternalEvent>,
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
            downloads: DownloadWorkers::new(),
            queue: MutablePriorityQueue::new(),
            probes: MetadataProbePool::new(config.tuning.probe_concurrency),
            retry: RetryBook::new(config.tuning.retry),
            pressure: StorePressure::new(config.tuning.store_pressure_pause),
            pending_demand: None,
            ctx: TransferContext {
                client: config.client,
                store: config.store,
                events: events_sender,
                timeouts: TransferTimeouts::default(),
                network: config.network,
            },
            cache: config.cache,
            commands,
            demand,
            events,
        }
    }
}
