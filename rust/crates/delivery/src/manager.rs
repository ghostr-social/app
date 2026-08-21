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
mod capability;
mod completion;
mod completion_decision;
mod completion_observability;
pub(crate) mod concurrency;
pub(crate) mod cooldown_timers;
mod create;
pub mod failure;
mod focus_lease;
pub(crate) mod immediate_replan;
mod independent_objects;
pub(crate) mod inflight;
mod integrity;
mod observability;
pub(crate) mod origin_admission;
pub(crate) mod plan;
mod playback;
mod policy_eviction;
mod presentation;
pub(crate) mod pressure;
mod probe_completion;
pub(crate) mod qoe;
pub(crate) mod quality;
pub(crate) mod reconcile;
mod reconcile_transfers;
pub(crate) mod reconcile_warp;
pub(crate) mod reliability;
mod request_gate;
mod reset;
mod response_observation;
pub(crate) mod response_open;
pub mod retry;
mod retry_completion;
pub(crate) mod selected_commit;
mod startup;
pub(crate) mod state;
pub(crate) mod stats;
#[cfg(test)]
mod testing;
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
use crate::manager::capability::CapabilityKeeper;
use crate::manager::cooldown_timers::CooldownTimers;
use crate::manager::immediate_replan::ImmediateReplan;
use crate::manager::independent_objects::IndependentObjects;
use crate::manager::pressure::StorePressure;
use crate::manager::qoe::QoeKeeper;
use crate::manager::reliability::ReliabilityKeeper;
use crate::manager::response_open::ResponseOpenReceiver;
use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use crate::manager::stats::StatsKeeper;
use crate::manager::timeline::TimelineCoordinator;
use crate::manager::traffic::TrafficInbox;
use crate::manager::transfers::{InternalEvent, TransferContext};
use crate::manager::wake_lane::WakeCursor;
use crate::manager::workers::DownloadWorkers;
use crate::mutable_priority_queue::MutablePriorityQueue;
use crate::playback_demand::DemandReceiver;
use crate::probe::pool::MetadataProbePool;
use crate::segmented::scheduler::SegmentedDelivery;
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::concurrency::AdaptiveConcurrency;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

pub(crate) use focus_lease::FocusedStoreLease;
pub use tuning::DeliveryTuning;

const TRAFFIC_MAILBOX_CAPACITY: usize = 64;

/// Everything the manager owns or reaches, as one typed object.
pub struct DeliveryManagerConfig {
    pub store: Arc<PartialRangeStore>,
    pub requests: MediaRequestExecutor,
    pub cache: CacheRegistry,
    pub segmented: SegmentedCache,
    pub network: NetworkThrottle,
    pub stats_path: PathBuf,
    pub params: EngineParams,
    pub level: DataUsageLevel,
    pub tuning: DeliveryTuning,
}

/// Starts the manager task and exposes adaptive candidate-demand changes.
pub fn start_delivery_manager_with_discovery_demand(
    config: DeliveryManagerConfig,
    demand: DemandReceiver,
) -> (DeliveryHandle, watch::Receiver<DiscoveryDemand>) {
    let (discovery_sender, discovery_updates) = watch::channel(DiscoveryDemand::Expand);
    let (handle, commands) = command_channel();
    tokio::spawn(run(config, commands, demand, Some(discovery_sender)));
    (handle, discovery_updates)
}

async fn run(
    config: DeliveryManagerConfig,
    commands: CommandReceiver,
    demand: DemandReceiver,
    discovery_watch: Option<watch::Sender<DiscoveryDemand>>,
) {
    let mut worker = DeliveryWorker::create(config, commands, demand).await;
    worker.spawn_capacity_replans();
    if let Some(sender) = discovery_watch {
        worker.state.publish_discovery_demand(sender);
    }
    while worker.step().await {}
    worker.keeper.save_now().await;
    let evidence = worker.state.catalog().evidence_state();
    worker.reliability.save_now(&evidence).await;
    worker.save_capability().await;
    worker.qoe.save_now().await;
}

pub(crate) struct DeliveryWorker {
    state: DeliveryState,
    keeper: StatsKeeper,
    reliability: ReliabilityKeeper,
    capability: CapabilityKeeper,
    qoe: QoeKeeper,
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
    responses: ResponseOpenReceiver,
    traffic: TrafficInbox,
    wake_cursor: WakeCursor,
    concurrency: AdaptiveConcurrency,
    additional_request_slot_demand: Option<bool>,
    max_requests_per_authority: Option<NonZeroUsize>,
    segmented: SegmentedDelivery,
    timelines: TimelineCoordinator,
    independent_objects: IndependentObjects,
    immediate_replan: ImmediateReplan,
    warp_planner: ghostr_engine::adaptive::WarpPlanner,
}
