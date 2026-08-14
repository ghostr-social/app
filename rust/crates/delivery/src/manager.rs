//! Event-driven delivery manager (plan Phase 1 step 7): replaces the
//! old one-second poll loop. Control and completion updates arrive over channels; every event
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
pub(crate) mod cooldown_timers;
pub mod failure;
mod focus_lease;
pub(crate) mod inflight;
pub(crate) mod plan;
mod policy_eviction;
pub(crate) mod pressure;
mod probe_completion;
pub(crate) mod quality;
pub(crate) mod reconcile;
mod reconcile_transfers;
mod reset;
pub mod retry;
mod retry_completion;
pub(crate) mod state;
pub(crate) mod stats;
pub(crate) mod time;
pub(crate) mod timeline;
pub(crate) mod traffic;
pub(crate) mod transfers;
mod tuning;
pub(crate) mod wake;
pub(crate) mod wake_lane;
pub(crate) mod wake_select;
pub(crate) mod workers;

use crate::cache_registry::CacheRegistry;
use crate::debug::network::NetworkThrottle;
use crate::delivery_events::{command_channel, CommandReceiver, DeliveryHandle};
use crate::demand_leases::DemandLeases;
use crate::manager::cooldown_timers::CooldownTimers;
use crate::manager::pressure::StorePressure;
use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use crate::manager::stats::StatsKeeper;
use crate::manager::traffic::{channel as traffic_channel, TrafficInbox};
use crate::manager::transfers::{InternalEvent, TransferContext};
use crate::manager::wake_lane::WakeCursor;
use crate::manager::workers::DownloadWorkers;
use crate::mutable_priority_queue::MutablePriorityQueue;
use crate::playback_demand::DemandReceiver;
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::concurrency::AdaptiveConcurrency;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

pub(crate) use focus_lease::FocusedStoreLease;
pub use tuning::DeliveryTuning;

const TRAFFIC_MAILBOX_CAPACITY: usize = 64;

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

/// Starts the manager task and exposes adaptive candidate-demand changes.
pub fn start_delivery_manager_with_discovery_demand<C>(
    config: DeliveryManagerConfig<C>,
    demand: DemandReceiver,
) -> (DeliveryHandle, watch::Receiver<DiscoveryDemand>)
where
    C: MediaHttpRequests + 'static,
{
    let (discovery_sender, discovery_updates) = watch::channel(DiscoveryDemand::Expand);
    let (handle, commands) = command_channel();
    tokio::spawn(run(config, commands, demand, Some(discovery_sender)));
    (handle, discovery_updates)
}

async fn run<C>(
    config: DeliveryManagerConfig<C>,
    commands: CommandReceiver,
    demand: DemandReceiver,
    discovery_watch: Option<watch::Sender<DiscoveryDemand>>,
) where
    C: MediaHttpRequests + 'static,
{
    let mut worker = DeliveryWorker::create(config, commands, demand).await;
    worker.spawn_capacity_replans();
    if let Some(sender) = discovery_watch {
        worker.state.publish_discovery_demand(sender);
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
    cooldown_timers: CooldownTimers,
    pressure: StorePressure,
    focus_lease: FocusedStoreLease,
    demand_leases: DemandLeases,
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
            cooldown_timers: CooldownTimers::default(),
            pressure: StorePressure::new(config.tuning.store_pressure_pause),
            focus_lease: FocusedStoreLease::default(),
            demand_leases: DemandLeases::default(),
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
