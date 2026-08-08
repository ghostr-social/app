//! A live delivery manager over a temp store. Routes that reach the
//! manager — clearing, for one — answer only while it is running, so a
//! bare command channel will not stand in for it.

use super::{media_client, temp_directory};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::manager::{
    start_delivery_manager_with_modes, DeliveryManagerConfig, DeliveryTuning,
};
use ghostr_delivery::playback_demand::{demand_channel, DemandSender};
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_partial_store::partial_range_store::{capacity::StoreCapacity, PartialRangeStore};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DeliveryFixture {
    pub handle: DeliveryHandle,
    pub demand: DemandSender,
    pub store: Arc<PartialRangeStore>,
    pub cache: ServablePosts,
    pub network: NetworkThrottle,
    pub root: PathBuf,
}

pub fn start_delivery(prefix: &str) -> DeliveryFixture {
    let root = temp_directory(prefix);
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let cache = ServablePosts::new();
    let network = NetworkThrottle::new();
    let (demand, demand_receiver) = demand_channel();
    let (handle, _modes) = start_delivery_manager_with_modes(
        DeliveryManagerConfig {
            store: store.clone(),
            client: media_client(),
            cache: cache.clone(),
            network: network.clone(),
            stats_path: root.join("host_stats.json"),
            params: EngineParams::default(),
            level: DataUsageLevel::Balanced,
            tuning: DeliveryTuning::default(),
        },
        demand_receiver,
    );
    DeliveryFixture {
        handle,
        demand,
        store,
        cache,
        network,
        root,
    }
}
