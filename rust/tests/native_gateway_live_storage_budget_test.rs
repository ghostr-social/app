mod support;

use rust_lib_ghostr::video::gateway_runtime::{GatewayConfiguration, GatewayRuntime};
use support::fixtures::temp_directory;

#[tokio::test]
async fn gateway_runtime_applies_live_storage_budget_to_progressive_media() {
    let directory = temp_directory("ghostr-gateway-live-budget");
    let (_endpoint, runtime, _client, _modes) = GatewayRuntime::start(GatewayConfiguration {
        cache_directory: directory.clone(),
        relays: Vec::new(),
        max_parallel_downloads: 1,
        max_storage_bytes: 800,
    })
    .await
    .expect("gateway start");
    let progressive = runtime.progressive();
    let store = &progressive.store;
    store
        .write_range("older", 0, &[1; 400])
        .await
        .expect("older");
    store
        .write_range("newer", 0, &[2; 400])
        .await
        .expect("newer");

    runtime
        .set_storage_budget(400)
        .await
        .expect("shrink live budget");

    assert_eq!(store.present_ranges("older").await.expect("older"), vec![]);
    assert_eq!(
        store.present_ranges("newer").await.expect("newer"),
        vec![0..400]
    );
    assert!(runtime.set_storage_budget(0).await.is_err(), "zero budget");
    runtime
        .set_storage_budget(800)
        .await
        .expect("expand live budget");
    store
        .write_range("future", 0, &[3; 400])
        .await
        .expect("future");
    assert_eq!(
        store.present_ranges("future").await.expect("future"),
        vec![0..400]
    );
    std::fs::remove_dir_all(directory).ok();
}
