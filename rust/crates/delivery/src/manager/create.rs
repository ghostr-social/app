use super::{DeliveryManagerConfig, DeliveryWorker, TRAFFIC_MAILBOX_CAPACITY};
use crate::demand_leases::DemandLeases;
use crate::manager::capability::CapabilityKeeper;
use crate::manager::cooldown_timers::CooldownTimers;
use crate::manager::focus_lease::FocusedStoreLease;
use crate::manager::independent_objects::IndependentObjects;
use crate::manager::pressure::StorePressure;
use crate::manager::qoe::QoeKeeper;
use crate::manager::reliability::ReliabilityKeeper;
use crate::manager::response_open;
use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use crate::manager::stats::StatsKeeper;
use crate::manager::timeline::TimelineCoordinator;
use crate::manager::traffic::channel as traffic_channel;
use crate::manager::transfers::TransferContext;
use crate::manager::wake_lane::WakeCursor;
use crate::manager::workers::DownloadWorkers;
use crate::mutable_priority_queue::MutablePriorityQueue;
use crate::probe::pool::MetadataProbePool;
use crate::segmented::scheduler::SegmentedDelivery;
use ghostr_engine::concurrency::AdaptiveConcurrency;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;

impl DeliveryWorker {
    pub(super) async fn create<C>(
        config: DeliveryManagerConfig<C>,
        commands: crate::delivery_events::CommandReceiver,
        demand: crate::playback_demand::DemandReceiver,
    ) -> Self
    where
        C: MediaHttpRequests + 'static,
    {
        let (events_sender, events) = tokio::sync::mpsc::unbounded_channel();
        let timeouts = TransferTimeouts::default();
        let (response_opener, responses) = response_open::channel(timeouts.idle);
        let (traffic_publisher, traffic) =
            traffic_channel(events_sender.clone(), TRAFFIC_MAILBOX_CAPACITY);
        let reliability_path = config.stats_path.with_file_name("field_reliability.json");
        let (reliability, evidence) =
            ReliabilityKeeper::load(reliability_path, config.tuning.stats_debounce).await;
        let mut state = DeliveryState::new(config.params, config.level);
        state
            .catalog_mut()
            .replace_evidence_state(evidence, crate::manager::time::unix_time_ms());
        let capability_path = config.stats_path.with_file_name("client_capability.json");
        let (capability, profile) =
            CapabilityKeeper::load(capability_path, config.tuning.stats_debounce).await;
        state.replace_client_capabilities(profile);
        let concurrency = AdaptiveConcurrency::new(1, state.concurrency());
        let segmented = SegmentedDelivery::new(config.segmented);
        let qoe_path = config.stats_path.with_file_name("qoe_stats.json");
        let qoe = QoeKeeper::load(
            qoe_path,
            config.tuning.stats_debounce,
            commands.evaluation(),
        )
        .await;
        let timelines = TimelineCoordinator::new(config.store.clone());
        Self {
            state,
            keeper: StatsKeeper::load(config.stats_path, config.tuning.stats_debounce).await,
            reliability,
            capability,
            qoe,
            downloads: DownloadWorkers::new(),
            queue: MutablePriorityQueue::new(),
            probes: MetadataProbePool::new(config.tuning.probe_concurrency),
            retry: RetryBook::new(config.tuning.retry),
            cooldown_timers: CooldownTimers::default(),
            pressure: StorePressure::new(config.tuning.store_pressure_pause),
            focus_lease: FocusedStoreLease::default(),
            demand_leases: DemandLeases::default(),
            ctx: TransferContext {
                client: std::sync::Arc::new(config.client),
                store: config.store,
                events: events_sender,
                responses: response_opener,
                timeouts,
                network: config.network,
                traffic: traffic_publisher,
            },
            cache: config.cache,
            commands,
            demand,
            events,
            responses,
            traffic,
            wake_cursor: WakeCursor::default(),
            concurrency,
            max_requests_per_authority: config.tuning.max_requests_per_authority,
            segmented,
            timelines,
            independent_objects: IndependentObjects::default(),
            warp_planner: ghostr_engine::adaptive::WarpPlanner::default(),
        }
    }
}
