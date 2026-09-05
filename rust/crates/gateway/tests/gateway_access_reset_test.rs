mod gateway_fixture;
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use std::sync::Arc;

#[tokio::test]
async fn reset_revokes_hls_sessions_without_restarting_the_gateway() {
    let root = gateway_fixture::temp_directory("access-reset");
    let (_, runtime, _) = GatewayRuntime::start(
        GatewayConfiguration {
            cache_directory: root.clone(),
            relays: Vec::new(),
            max_parallel_downloads: 2,
            max_storage_bytes: 1_024,
            internet_data_limit: ghostr_net::internet_allowance::InternetDataLimit::Unlimited,
            network_status: ghostr_delivery::delivery_events::DeliveryNetworkStatus::unavailable(),
            device_integration_origin: None,
        },
        Arc::new(client_with_event_cache()),
    )
    .await
    .expect("fixture");
    let old = runtime
        .acquire_unprepared_hls(vec!["https://media.example/stream.m3u8".to_owned()])
        .await
        .expect("fixture");
    runtime.reset_playback_access().await.expect("fixture");
    assert!(!runtime.release_hls(old.id.as_str()).await);
    let fresh = runtime
        .acquire_unprepared_hls(vec!["https://media.example/stream.m3u8".to_owned()])
        .await
        .expect("fixture");
    assert_ne!(fresh.id, old.id);
    assert!(runtime.release_hls(fresh.id.as_str()).await);
    std::fs::remove_dir_all(root).expect("fixture");
}
