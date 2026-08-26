//! The delivery manager publishes resource-driven candidate demand to discovery.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::{media_client, temp_directory};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::manager::{
    start_delivery_manager_with_discovery_demand, DeliveryManagerConfig, DeliveryTuning,
};
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::timeout;

#[tokio::test]
async fn delivery_manager_publishes_adaptive_discovery_demand() {
    let root = temp_directory("ghostr-mode-watch");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let (_demand, demand_receiver) = demand_channel();
    let config = DeliveryManagerConfig {
        store,
        requests: media_client(),
        cache: ServablePosts::new(),
        segmented: SegmentedCache::new(),
        network: NetworkThrottle::new(),
        network_status: ghostr_delivery::delivery_events::DeliveryNetworkStatus::unavailable(),
        stats_path: root.join("host_stats.json"),
        params: EngineParams::default(),
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
        transform: None,
    };
    let (handle, mut discovery_demand) =
        start_delivery_manager_with_discovery_demand(config, demand_receiver);
    assert_eq!(
        *discovery_demand.borrow(),
        DiscoveryDemand::Expand,
        "an empty candidate supply needs expansion"
    );

    let unreachable = "http://127.0.0.1:9/video.mp4";
    handle.update_focus(focus_now(
        vec![
            sized_item("aa11", unreachable, 64, 1_000),
            sized_item("bb22", unreachable, 64, 1_000),
            sized_item("cc33", unreachable, 64, 1_000),
            sized_item("dd44", unreachable, 64, 1_000),
        ],
        0,
        0,
    ));
    timeout(Duration::from_secs(5), discovery_demand.changed())
        .await
        .expect("a waiting unserved candidate should publish a hold")
        .expect("manager should stay alive");
    assert_eq!(*discovery_demand.borrow(), DiscoveryDemand::Hold);

    handle.update_focus(focus_now(Vec::new(), 0, 0));
    timeout(Duration::from_secs(5), discovery_demand.changed())
        .await
        .expect("empty supply should request expansion")
        .expect("manager should stay alive");
    assert_eq!(*discovery_demand.borrow(), DiscoveryDemand::Expand);
    handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&root).ok();
}
