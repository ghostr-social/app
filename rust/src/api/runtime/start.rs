//! Discovery runtime composition kept separate from its live control surface.

use crate::api::feed::state::FeedState;
use crate::api::runtime::configuration;
use crate::api::runtime::discovery::{
    pump_outcomes, DiscoveryBoot, DiscoveryRuntime, OutcomeSinks, SharedFeedState,
};
use crate::discovery::cache::EventCache;
use crate::discovery::execution::relay_executor::RelayPlanExecutor;
use crate::discovery::outbox::bootstrap::OutboxBootstrap;
use crate::discovery::outbox::directory::OutboxDirectory;
use crate::discovery::outbox::directory::SharedOutboxDirectory;
use crate::discovery::retrieval_types::RetrievalOutcome;
use crate::discovery::scheduler::{
    start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig,
};
use crate::engine::adaptive::DiscoveryDemand;
use crate::engine::DataUsageLevel;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch, RwLock};

pub(super) async fn start(boot: DiscoveryBoot, cache: Option<Arc<EventCache>>) -> DiscoveryRuntime {
    let relay_pool = configuration::initialize_relay_pool(
        std::sync::Arc::clone(&boot.client),
        boot.bootstrap.clone(),
        boot.search_relays.clone(),
    )
    .await;
    let outbox = shared_outbox(boot.bootstrap);
    let executor = RelayPlanExecutor::with_owner(
        std::sync::Arc::clone(&relay_pool),
        boot.search_relays,
        std::sync::Arc::clone(&outbox),
        DataUsageLevel::Balanced,
    );
    let executor = match cache {
        Some(cache) => executor.with_cache(cache),
        None => executor,
    };
    let pipeline = DiscoveryPipeline::start(
        executor.clone(),
        std::sync::Arc::clone(&outbox),
        boot.demand,
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
        demand: watch::Receiver<DiscoveryDemand>,
        candidates: Option<ghostr_delivery::delivery_events::DeliveryHandle>,
    ) -> Self {
        let (sender, outcomes) = mpsc::unbounded_channel();
        let handle = scheduler(executor.clone(), demand, sender.clone());
        let state = Arc::new(Mutex::new(FeedState::new()));
        let bootstrap = Arc::new(OutboxBootstrap::new(Arc::new(executor), outbox, sender));
        spawn_pump(
            std::sync::Arc::clone(&state),
            std::sync::Arc::clone(&bootstrap),
            candidates,
            outcomes,
        );
        Self {
            handle,
            state,
            bootstrap,
        }
    }
}

fn scheduler(
    executor: RelayPlanExecutor,
    demand: watch::Receiver<DiscoveryDemand>,
    outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
) -> DiscoveryHandle {
    start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(executor),
        level: DataUsageLevel::Balanced,
        demand,
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
