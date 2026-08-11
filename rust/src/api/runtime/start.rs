//! Discovery runtime composition kept separate from its live control surface.

use crate::api::feed::state::FeedState;
use crate::api::runtime::configuration;
use crate::api::runtime::discovery::{
    pump_outcomes, DiscoveryBoot, DiscoveryRuntime, OutcomeSinks, SharedFeedState,
};
use crate::discovery::execution::relay_executor::RelayPlanExecutor;
use crate::discovery::outbox::bootstrap::OutboxBootstrap;
use crate::discovery::outbox::directory::OutboxDirectory;
use crate::discovery::outbox::directory::SharedOutboxDirectory;
use crate::discovery::retrieval_types::RetrievalOutcome;
use crate::discovery::scheduler::{
    start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig,
};
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch, RwLock};

pub(crate) async fn start(boot: DiscoveryBoot) -> DiscoveryRuntime {
    let relay_pool = configuration::initialize_relay_pool(
        boot.client.clone(),
        boot.bootstrap.clone(),
        boot.search_relays.clone(),
    )
    .await;
    let outbox = shared_outbox(boot.bootstrap);
    let executor = RelayPlanExecutor::with_owner(
        relay_pool.clone(),
        boot.search_relays,
        outbox.clone(),
        DataUsageLevel::Balanced,
    );
    let pipeline = DiscoveryPipeline::start(
        executor.clone(),
        outbox.clone(),
        boot.modes,
        boot.candidates,
    );
    DiscoveryRuntime {
        handle: pipeline.handle,
        state: pipeline.state,
        client: boot.client,
        outbox,
        executor,
        bootstrap: pipeline.bootstrap,
        relay_pool,
    }
}

struct DiscoveryPipeline {
    handle: DiscoveryHandle,
    state: SharedFeedState,
    bootstrap: Arc<OutboxBootstrap>,
}

impl DiscoveryPipeline {
    fn start(
        executor: RelayPlanExecutor,
        outbox: SharedOutboxDirectory,
        modes: watch::Receiver<Mode>,
        candidates: Option<ghostr_delivery::delivery_events::DeliveryHandle>,
    ) -> Self {
        let (sender, outcomes) = mpsc::unbounded_channel();
        let handle = scheduler(executor.clone(), modes, sender.clone());
        let state = Arc::new(Mutex::new(FeedState::new()));
        let bootstrap = Arc::new(OutboxBootstrap::new(Arc::new(executor), outbox, sender));
        spawn_pump(state.clone(), bootstrap.clone(), candidates, outcomes);
        Self {
            handle,
            state,
            bootstrap,
        }
    }
}

fn scheduler(
    executor: RelayPlanExecutor,
    modes: watch::Receiver<Mode>,
    outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
) -> DiscoveryHandle {
    start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(executor),
        level: DataUsageLevel::Balanced,
        modes,
        outcomes,
    })
}

fn spawn_pump(
    state: SharedFeedState,
    bootstrap: Arc<OutboxBootstrap>,
    candidates: Option<ghostr_delivery::delivery_events::DeliveryHandle>,
    outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
) {
    tokio::spawn(pump_outcomes(
        OutcomeSinks {
            state,
            bootstrap,
            candidates,
        },
        outcomes,
    ));
}

fn shared_outbox(bootstrap: Vec<String>) -> SharedOutboxDirectory {
    Arc::new(RwLock::new(OutboxDirectory::new(bootstrap)))
}
