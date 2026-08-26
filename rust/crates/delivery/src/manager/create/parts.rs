use super::{initial_channels, InitialPolicy};
use crate::demand_leases::DemandLeases;
use crate::manager::cooldown_timers::CooldownTimers;
use crate::manager::focus_lease::FocusedStoreLease;
use crate::manager::independent_objects::IndependentObjects;
use crate::manager::pressure::StorePressure;
use crate::manager::response_open;
use crate::manager::retry::RetryBook;
use crate::manager::timeline::TimelineCoordinator;
use crate::manager::traffic::TrafficInbox;
use crate::manager::transfers::{InternalEvent, TransferContext};
use crate::manager::wake_lane::WakeCursor;
use crate::manager::workers::DownloadWorkers;
use crate::manager::{DeliveryManagerConfig, DeliveryWorker};
use crate::mutable_priority_queue::MutablePriorityQueue;
use crate::probe::pool::MetadataProbePool;
use crate::segmented::scheduler::SegmentedDelivery;
use tokio::sync::mpsc;

pub(super) struct WorkerParts {
    policy: InitialPolicy,
    cache: crate::cache_registry::CacheRegistry,
    commands: crate::delivery_events::CommandReceiver,
    demand: crate::playback_demand::DemandReceiver,
    events: mpsc::UnboundedReceiver<InternalEvent>,
    responses: response_open::ResponseOpenReceiver,
    traffic: TrafficInbox,
    context: TransferContext,
    tuning: crate::manager::DeliveryTuning,
    segmented: SegmentedDelivery,
    segmented_invalidations: tokio::sync::watch::Receiver<u64>,
    timelines: TimelineCoordinator,
    transforms: crate::manager::transforms::TransformJobs,
    resources: crate::manager::resource_control::ResourceControl,
}

impl WorkerParts {
    pub(super) async fn load(
        config: DeliveryManagerConfig,
        commands: crate::delivery_events::CommandReceiver,
        demand: crate::playback_demand::DemandReceiver,
        resources: crate::manager::resource_control::ResourceControl,
    ) -> Self {
        let channels = initial_channels();
        let policy = InitialPolicy::load(&config, &commands).await;
        let network_status =
            crate::delivery_events::DeliveryNetworkStatusReader::new(config.network_status);
        let segmented_invalidations = config.segmented.invalidation_receiver();
        let timelines = TimelineCoordinator::new(std::sync::Arc::clone(&config.store));
        let transforms = crate::manager::transforms::TransformJobs::new(
            config.transform.clone(),
            channels.events_sender.clone(),
            resources.clone(),
        );
        let context = TransferContext {
            requests: config.requests,
            store: config.store,
            events: channels.events_sender,
            responses: channels.response_opener,
            timeouts: channels.timeouts,
            network: config.network,
            traffic: channels.traffic_publisher,
            network_status,
        };
        Self {
            policy,
            cache: config.cache,
            commands,
            demand,
            events: channels.events,
            responses: channels.responses,
            traffic: channels.traffic,
            context,
            tuning: config.tuning,
            segmented: SegmentedDelivery::new(config.segmented),
            segmented_invalidations,
            timelines,
            transforms,
            resources,
        }
    }
}

impl From<WorkerParts> for DeliveryWorker {
    fn from(parts: WorkerParts) -> Self {
        Self {
            state: parts.policy.state,
            keeper: parts.policy.keeper,
            reliability: parts.policy.reliability,
            capability: parts.policy.capability,
            qoe: parts.policy.qoe,
            downloads: DownloadWorkers::new(),
            queue: MutablePriorityQueue::new(),
            probes: MetadataProbePool::new(parts.tuning.probe_concurrency),
            retry: RetryBook::new(parts.tuning.retry),
            cooldown_timers: CooldownTimers::default(),
            pressure: StorePressure::new(parts.tuning.store_pressure_pause),
            focus_lease: FocusedStoreLease::default(),
            hedge_tail_timers: Default::default(),
            demand_leases: DemandLeases::default(),
            ctx: parts.context,
            cache: parts.cache,
            commands: parts.commands,
            demand: parts.demand,
            events: parts.events,
            responses: parts.responses,
            traffic: parts.traffic,
            control_interval: crate::manager::control_interval::new_at(parts.resources.origin()),
            wake_cursor: WakeCursor::default(),
            concurrency: parts.policy.concurrency,
            additional_request_slot_demand: None,
            max_requests_per_authority: parts.tuning.max_requests_per_authority,
            segmented: parts.segmented,
            segmented_invalidations: parts.segmented_invalidations,
            timelines: parts.timelines,
            independent_objects: IndependentObjects::default(),
            whole_body_limits: Default::default(),
            transforms: parts.transforms,
            immediate_replan: Default::default(),
            network_refill_timer: Default::default(),
            resources: parts.resources,
            warp_planner: ghostr_engine::adaptive::WarpPlanner::default(),
        }
    }
}
