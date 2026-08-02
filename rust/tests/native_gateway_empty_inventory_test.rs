mod support;

use rust_lib_ghostr::video::video::{ffi_get_discovered_videos, ffi_start_server};
use support::fixtures::temp_directory;

#[tokio::test]
async fn exposes_an_empty_inventory_after_an_offline_gateway_start() {
    let directory = temp_directory("ghostr-gateway-empty");
    ffi_start_server(
        directory.to_string_lossy().to_string(),
        1,
        1024,
        String::new(),
    )
    .await
    .expect("gateway start");

    assert!(ffi_get_discovered_videos().await.is_empty());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
