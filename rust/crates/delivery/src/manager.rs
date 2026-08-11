//! Event-driven delivery manager (plan Phase 1 step 7): replaces the
//! old one-second poll loop. Focus updates, config changes, gateway
//! demand, and transfer completions arrive over channels; every event
//! triggers one replanning pass — there is no periodic wake-up.
//!
//! The parts of that pass live beside this file: `reconcile` decides
//! what should be in flight, `transfers` and `workers` run it,
//! `completion` and `probe_completion` absorb the results, and
//! `retry`/`failure`/`pressure` decide what a failure means.

pub(crate) mod admission;
mod cache;
mod completion;
pub(crate) mod concurrency;
pub mod failure;
pub(crate) mod inflight;
pub(crate) mod plan;
pub(crate) mod pressure;
mod probe_completion;
pub(crate) mod reconcile;
mod reset;
pub mod retry;
pub(crate) mod state;
pub(crate) mod stats;
pub(crate) mod traffic;
pub(crate) mod transfers;
mod wake;
pub(crate) mod wake_lane;
mod wake_select;
pub(crate) mod workers;

use crate::cache_registry::CacheRegistry;
use crate::debug::network::NetworkThrottle;
use crate::delivery_events::{command_channel, CommandReceiver, DeliveryHandle};
use crate::manager::pressure::StorePressure;
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::manager::stats::StatsKeeper;
use crate::manager::traffic::{channel as traffic_channel, TrafficInbox};
use crate::manager::transfers::{InternalEvent, TransferContext};
use crate::manager::wake_lane::WakeCursor;
use crate::manager::workers::DownloadWorkers;
use crate::mutable_priority_queue::MutablePriorityQueue;
use crate::playback_demand::{DemandReceiver, DemandSignal};
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::concurrency::AdaptiveConcurrency;
use ghostr_engine::inventory_controller::Mode;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::capacity::DEFAULT_RECHECK;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

const TRAFFIC_MAILBOX_CAPACITY: usize = 64;

/// Operational knobs outside the engine's tuning table.
#[derive(Clone, Copy, Debug)]
pub struct DeliveryTuning {
    /// Concurrent HEAD probes for unknown-size posts.
    pub probe_concurrency: usize,
    /// Backoff ladder and give-up budgets for failing sources.
    pub retry: RetryPolicy,
    /// Quiet period before persisting the host-stats snapshot.
    pub stats_debounce: Duration,
    /// Delay before one free-space recheck after a store refusal. Later
    /// retries remain parked until a real capacity event.
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
pub struct DeliveryManagerConfig<C = MediaHttpClient> {
    pub store: Arc<PartialRangeStore>,
    pub client: C,
    pub cache: CacheRegistry,
    pub network: NetworkThrottle,
    pub stats_path: PathBuf,
    pub params: EngineParams,
    pub level: DataUsageLevel,
    pub tuning: DeliveryTuning,
}

/// Starts the manager task and exposes the inventory mode transitions the
/// discovery control loop subscribes to. The receiver starts at
/// [`Mode::Hunger`], matching a fresh controller.
pub fn start_delivery_manager_with_modes<C>(
    config: DeliveryManagerConfig<C>,
    demand: DemandReceiver,
) -> (DeliveryHandle, watch::Receiver<Mode>)
where
    C: MediaHttpRequests + 'static,
{
    let (modes, mode_updates) = watch::channel(Mode::Hunger);
    let (handle, commands) = command_channel();
    tokio::spawn(run(config, commands, demand, Some(modes)));
    (handle, mode_updates)
}

async fn run<C>(
    config: DeliveryManagerConfig<C>,
    commands: CommandReceiver,
    demand: DemandReceiver,
    modes: Option<watch::Sender<Mode>>,
) where
    C: MediaHttpRequests + 'static,
{
    let mut worker = DeliveryWorker::create(config, commands, demand).await;
    if let Some(sender) = modes {
        worker.state.publish_modes(sender);
    }
    while worker.step().await {}
    worker.keeper.save_now().await;
}

pub(crate) struct DeliveryWorker {
    state: DeliveryState,
    keeper: StatsKeeper,
    downloads: DownloadWorkers,
    queue: MutablePriorityQueue,
    probes: MetadataProbePool,
    retry: RetryBook,
    pressure: StorePressure,
    pending_demand: Option<DemandSignal>,
    ctx: TransferContext,
    cache: CacheRegistry,
    commands: CommandReceiver,
    demand: DemandReceiver,
    events: mpsc::UnboundedReceiver<InternalEvent>,
    traffic: TrafficInbox,
    wake_cursor: WakeCursor,
    concurrency: AdaptiveConcurrency,
}

impl DeliveryWorker {
    async fn create<C>(
        config: DeliveryManagerConfig<C>,
        commands: CommandReceiver,
        demand: DemandReceiver,
    ) -> Self
    where
        C: MediaHttpRequests + 'static,
    {
        let (events_sender, events) = mpsc::unbounded_channel();
        let (traffic_publisher, traffic) =
            traffic_channel(events_sender.clone(), TRAFFIC_MAILBOX_CAPACITY);
        let state = DeliveryState::new(config.params, config.level);
        let concurrency = AdaptiveConcurrency::new(1, state.concurrency());
        Self {
            state,
            keeper: StatsKeeper::load(config.stats_path, config.tuning.stats_debounce).await,
            downloads: DownloadWorkers::new(),
            queue: MutablePriorityQueue::new(),
            probes: MetadataProbePool::new(config.tuning.probe_concurrency),
            retry: RetryBook::new(config.tuning.retry),
            pressure: StorePressure::new(config.tuning.store_pressure_pause),
            pending_demand: None,
            ctx: TransferContext {
                client: Arc::new(config.client),
                store: config.store,
                events: events_sender,
                timeouts: TransferTimeouts::default(),
                network: config.network,
                traffic: traffic_publisher,
            },
            cache: config.cache,
            commands,
            demand,
            events,
            traffic,
            wake_cursor: WakeCursor::default(),
            concurrency,
        }
    }
}
