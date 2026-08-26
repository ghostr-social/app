//! Event-driven delivery manager (plan Phase 1 step 7).
//!
//! Control and completion updates arrive over channels. Every event triggers one replanning pass,
//! with one bounded control-interval wake for sampled feedback, replacing the old one-second loop.
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
mod config;
pub(super) mod control_interval;
mod cooldown_completion;
pub(crate) mod cooldown_timers;
mod create;
pub mod failure;
mod focus_lease;
mod focus_retry;
pub(crate) mod hedge_tail;
pub(crate) mod immediate_replan;
mod independent_objects;
pub(crate) mod inflight;
mod integrity;
pub(crate) mod network_refill_timer;
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
pub(crate) mod resource_control;
mod response_generation;
mod response_observation;
pub(crate) mod response_open;
pub mod retry;
mod retry_completion;
mod segmented;
pub(crate) mod selected_commit;
mod startup;
#[cfg(test)]
mod startup_snapshot_test;
pub(crate) mod state;
pub(crate) mod stats;
#[cfg(test)]
mod testing;
pub(crate) mod time;
pub(crate) mod timeline;
pub(crate) mod traffic;
pub(crate) mod transfers;
mod transforms;
mod tuning;
pub(crate) mod wake;
pub(crate) mod wake_lane;
pub(crate) mod wake_select;
mod whole_body_limits;
pub(crate) mod workers;

use crate::cache_registry::CacheRegistry;
use crate::delivery_events::{command_channel, CommandReceiver, DeliveryHandle};
use crate::demand_leases::DemandLeases;
use crate::manager::capability::CapabilityKeeper;
use crate::manager::cooldown_timers::CooldownTimers;
use crate::manager::hedge_tail::HedgeTailTimers;
use crate::manager::immediate_replan::ImmediateReplan;
use crate::manager::independent_objects::IndependentObjects;
use crate::manager::network_refill_timer::NetworkRefillTimer;
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
use crate::manager::whole_body_limits::WholeBodyLimits;
use crate::manager::workers::DownloadWorkers;
use crate::mutable_priority_queue::MutablePriorityQueue;
use crate::playback_demand::DemandReceiver;
use crate::probe::pool::MetadataProbePool;
use crate::segmented::scheduler::SegmentedDelivery;
use core::num::NonZeroUsize;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::concurrency::AdaptiveConcurrency;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

pub use config::DeliveryManagerConfig;
pub(crate) use focus_lease::FocusedStoreLease;
pub use tuning::DeliveryTuning;

const TRAFFIC_MAILBOX_CAPACITY: usize = 64;

/// Starts the manager task and exposes adaptive candidate-demand changes.
pub fn start_delivery_manager_with_discovery_demand(
    config: DeliveryManagerConfig,
    demand: DemandReceiver,
) -> (DeliveryHandle, watch::Receiver<DiscoveryDemand>) {
    let (discovery_sender, discovery_updates) = watch::channel(DiscoveryDemand::Expand);
    let (handle, commands) = command_channel();
    let resources =
        resource_control::ResourceControl::bootstrap(&config, tokio::time::Instant::now());
    let observer = Arc::new(resources.clone());
    assert!(
        config.requests.install_resource_observer(observer),
        "a delivery manager must install its sole request resource observer"
    );
    tokio::spawn(async move {
        let worker = DeliveryWorker::create(config, commands, demand, resources).await;
        Box::pin(run(worker, Some(discovery_sender))).await;
    });
    (handle, discovery_updates)
}

async fn run(mut worker: DeliveryWorker, discovery_watch: Option<watch::Sender<DiscoveryDemand>>) {
    worker.spawn_capacity_replans();
    if let Some(sender) = discovery_watch {
        worker.state.publish_discovery_demand(sender);
    }
    while Box::pin(worker.step()).await {}
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
    hedge_tail_timers: HedgeTailTimers,
    demand_leases: DemandLeases,
    ctx: TransferContext,
    cache: CacheRegistry,
    commands: CommandReceiver,
    demand: DemandReceiver,
    events: mpsc::UnboundedReceiver<InternalEvent>,
    responses: ResponseOpenReceiver,
    traffic: TrafficInbox,
    control_interval: tokio::time::Interval,
    wake_cursor: WakeCursor,
    concurrency: AdaptiveConcurrency,
    additional_request_slot_demand: Option<bool>,
    max_requests_per_authority: Option<NonZeroUsize>,
    segmented: SegmentedDelivery,
    segmented_invalidations: watch::Receiver<u64>,
    timelines: TimelineCoordinator,
    independent_objects: IndependentObjects,
    whole_body_limits: WholeBodyLimits,
    transforms: transforms::TransformJobs,
    immediate_replan: ImmediateReplan,
    network_refill_timer: NetworkRefillTimer,
    resources: resource_control::ResourceControl,
    warp_planner: ghostr_engine::adaptive::WarpPlanner,
}
