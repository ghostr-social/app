use super::{DeliveryManagerConfig, DeliveryWorker, TRAFFIC_MAILBOX_CAPACITY};
use crate::demand_leases::DemandLeases;
use crate::manager::capability::CapabilityKeeper;
use crate::manager::concurrency::RequestConcurrencyLimits;
use crate::manager::cooldown_timers::CooldownTimers;
use crate::manager::focus_lease::FocusedStoreLease;
use crate::manager::independent_objects::IndependentObjects;
use crate::manager::pressure::StorePressure;
use crate::manager::qoe::QoeKeeper;
use crate::manager::reliability::ReliabilityKeeper;
use crate::manager::request_gate::apply_request_limits;
use crate::manager::response_open;
use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use crate::manager::stats::StatsKeeper;
use crate::manager::timeline::TimelineCoordinator;
use crate::manager::traffic::channel as traffic_channel;
use crate::manager::traffic::{TrafficInbox, TrafficPublisher};
use crate::manager::transfers::{InternalEvent, TransferContext};
use crate::manager::wake_lane::WakeCursor;
use crate::manager::workers::DownloadWorkers;
use crate::mutable_priority_queue::MutablePriorityQueue;
use crate::probe::pool::MetadataProbePool;
use crate::segmented::scheduler::SegmentedDelivery;
use ghostr_engine::concurrency::AdaptiveConcurrency;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use tokio::sync::mpsc;

struct InitialChannels {
    events_sender: mpsc::UnboundedSender<InternalEvent>,
    events: mpsc::UnboundedReceiver<InternalEvent>,
    response_opener: response_open::ResponseOpener,
    responses: response_open::ResponseOpenReceiver,
    traffic_publisher: TrafficPublisher,
    traffic: TrafficInbox,
    timeouts: TransferTimeouts,
}

struct InitialPolicy {
    state: DeliveryState,
    keeper: StatsKeeper,
    reliability: ReliabilityKeeper,
    capability: CapabilityKeeper,
    qoe: QoeKeeper,
    concurrency: AdaptiveConcurrency,
}

impl InitialPolicy {
    async fn load(
        config: &DeliveryManagerConfig,
        commands: &crate::delivery_events::CommandReceiver,
    ) -> Self {
        let reliability_path = config.stats_path.with_file_name("field_reliability.json");
        let (reliability, evidence) =
            ReliabilityKeeper::load(reliability_path, config.tuning.stats_debounce).await;
        let mut state = DeliveryState::new(config.params, config.level);
        state.apply_network_status(config.network_status);
        state.configure_transform(config.transform.as_ref().map(|backend| backend.profile()));
        let limits = RequestConcurrencyLimits::resolve(
            state.concurrency(),
            config.tuning.max_requests_per_authority,
            config.network.profile().max_connections_per_host,
        );
        apply_request_limits(&config.requests, limits);
        state
            .catalog_mut()
            .replace_evidence_state(evidence, crate::manager::time::unix_time_ms());
        let capability_path = config.stats_path.with_file_name("client_capability.json");
        let (capability, profile) =
            CapabilityKeeper::load(capability_path, config.tuning.stats_debounce).await;
        state.replace_client_capabilities(profile);
        let concurrency = AdaptiveConcurrency::new(1, state.concurrency());
        let qoe_path = config.stats_path.with_file_name("qoe_stats.json");
        let qoe = QoeKeeper::load(
            qoe_path,
            config.tuning.stats_debounce,
            commands.evaluation(),
        )
        .await;
        let keeper =
            StatsKeeper::load(config.stats_path.clone(), config.tuning.stats_debounce).await;
        Self {
            state,
            keeper,
            reliability,
            capability,
            qoe,
            concurrency,
        }
    }
}

fn initial_channels() -> InitialChannels {
    let (events_sender, events) = mpsc::unbounded_channel();
    let timeouts = TransferTimeouts::default();
    let (response_opener, responses) = response_open::channel(timeouts.idle);
    let (traffic_publisher, traffic) =
        traffic_channel(events_sender.clone(), TRAFFIC_MAILBOX_CAPACITY);
    InitialChannels {
        events_sender,
        events,
        response_opener,
        responses,
        traffic_publisher,
        traffic,
        timeouts,
    }
}

impl DeliveryWorker {
    pub(super) async fn create(
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
        let segmented = SegmentedDelivery::new(config.segmented);
        let timelines = TimelineCoordinator::new(config.store.clone());
        let transforms = crate::manager::transforms::TransformJobs::new(
            config.transform.clone(),
            channels.events_sender.clone(),
            resources.clone(),
        );
        Self {
            state: policy.state,
            keeper: policy.keeper,
            reliability: policy.reliability,
            capability: policy.capability,
            qoe: policy.qoe,
            downloads: DownloadWorkers::new(),
            queue: MutablePriorityQueue::new(),
            probes: MetadataProbePool::new(config.tuning.probe_concurrency),
            retry: RetryBook::new(config.tuning.retry),
            cooldown_timers: CooldownTimers::default(),
            pressure: StorePressure::new(config.tuning.store_pressure_pause),
            focus_lease: FocusedStoreLease::default(),
            hedge_tail_timers: Default::default(),
            demand_leases: DemandLeases::default(),
            ctx: TransferContext {
                requests: config.requests,
                store: config.store,
                events: channels.events_sender,
                responses: channels.response_opener,
                timeouts: channels.timeouts,
                network: config.network,
                traffic: channels.traffic_publisher,
                network_status,
            },
            cache: config.cache,
            commands,
            demand,
            events: channels.events,
            responses: channels.responses,
            traffic: channels.traffic,
            control_interval: crate::manager::control_interval::new_at(resources.origin()),
            wake_cursor: WakeCursor::default(),
            concurrency: policy.concurrency,
            additional_request_slot_demand: None,
            max_requests_per_authority: config.tuning.max_requests_per_authority,
            segmented,
            segmented_invalidations,
            timelines,
            independent_objects: IndependentObjects::default(),
            whole_body_limits: Default::default(),
            transforms,
            immediate_replan: Default::default(),
            network_refill_timer: Default::default(),
            resources,
            warp_planner: ghostr_engine::adaptive::WarpPlanner::default(),
        }
    }
}
