//! A live delivery manager over a temp store. Routes that reach the
//! manager — clearing, for one — answer only while it is running, so a
//! bare command channel will not stand in for it.

use super::{media_client, temp_directory};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::manager::{
    start_delivery_manager_with_discovery_demand, DeliveryManagerConfig, DeliveryTuning,
};
use ghostr_delivery::playback_demand::{demand_channel, DemandReceiver, DemandSender, DemandState};
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_partial_store::partial_range_store::{capacity::StoreCapacity, PartialRangeStore};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

pub struct DeliveryFixture {
    pub handle: DeliveryHandle,
    pub demand: DemandSender,
    pub store: Arc<PartialRangeStore>,
    pub cache: ServablePosts,
    pub segmented: SegmentedCache,
    pub network: NetworkThrottle,
    pub requests: MediaRequestExecutor,
    pub root: PathBuf,
    demands: Arc<StdMutex<Vec<DemandState>>>,
}

impl DeliveryFixture {
    pub fn demands(&self) -> Vec<DemandState> {
        self.demands.lock().expect("demand trace").clone()
    }
}

pub fn start_delivery(prefix: &str) -> DeliveryFixture {
    start_delivery_with_tuning(prefix, DeliveryTuning::default())
}

pub fn start_delivery_with_tuning(prefix: &str, tuning: DeliveryTuning) -> DeliveryFixture {
    let root = temp_directory(prefix);
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let cache = ServablePosts::new();
    let network = NetworkThrottle::new();
    let requests = media_client();
    let segmented = SegmentedCache::new();
    let (demand, demand_receiver) = demand_channel();
    let demands = Arc::new(StdMutex::new(Vec::new()));
    let demand_receiver = trace_demands(demand_receiver, demands.clone());
    let (handle, _discovery_demand) = start_delivery_manager_with_discovery_demand(
        DeliveryManagerConfig {
            store: store.clone(),
            requests: requests.clone(),
            cache: cache.clone(),
            segmented: segmented.clone(),
            network: network.clone(),
            stats_path: root.join("host_stats.json"),
            params: EngineParams::default(),
            level: DataUsageLevel::Balanced,
            tuning,
        },
        demand_receiver,
    );
    DeliveryFixture {
        handle,
        demand,
        store,
        cache,
        segmented,
        network,
        requests,
        root,
        demands,
    }
}

fn trace_demands(
    mut source: DemandReceiver,
    trace: Arc<StdMutex<Vec<DemandState>>>,
) -> DemandReceiver {
    let (forward, receiver) = demand_channel();
    tokio::spawn(async move {
        while let Some(signal) = source.recv().await {
            trace.lock().expect("demand trace").push(signal.clone());
            forward.emit(signal);
        }
    });
    receiver
}
