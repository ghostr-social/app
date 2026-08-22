//! Device pass 3: the progressive store measured 8 KB after every
//! launch while `host_stats.json` in the same directory persisted.
//! Starting the engine must keep what the last session prefetched.

mod support;

use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use ghostr_partial_store::partial_range_store::{capacity::StoreCapacity, PartialRangeStore};
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn starting_the_gateway_keeps_the_progressive_store() {
    let directory = temp_directory("ghostr-gateway-survival");
    let progressive = directory.join("progressive");
    seed_store(progressive).await;
    std::fs::write(directory.join("host_stats.json"), b"{}").expect("host model");

    let (_endpoint, runtime, _modes) = GatewayRuntime::start(
        GatewayConfiguration {
            cache_directory: directory.clone(),
            relays: Vec::new(),
            max_parallel_downloads: 1,
            max_storage_bytes: 1_048_576,
            network_status: ghostr_delivery::delivery_events::DeliveryNetworkStatus::unavailable(),
            device_integration_origin: None,
        },
        Arc::new(client_with_event_cache()),
    )
    .await
    .expect("gateway start");

    let store = &runtime.progressive().store;
    assert!(
        store.is_complete("clip").await.expect("completeness"),
        "the manifest the last run committed must be reloaded"
    );
    assert_eq!(
        store.read_range("clip", 0..16).await.expect("read"),
        Some(b"prefetched bytes".to_vec()),
        "the prefetched bytes are reused, not re-downloaded"
    );
    assert!(directory.join("host_stats.json").exists(), "host model");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

async fn seed_store(root: std::path::PathBuf) {
    let store = PartialRangeStore::with_capacity(
        root,
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(1_048_576),
    );
    store
        .write_range("clip", 0, b"prefetched bytes")
        .await
        .expect("bytes");
    store.set_total_len("clip", 16).await.expect("manifest");
}
