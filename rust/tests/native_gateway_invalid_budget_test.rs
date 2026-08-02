mod support;

use rust_lib_ghostr::video::video::ffi_start_server;
use support::fixtures::temp_directory;

#[tokio::test]
async fn rejects_a_zero_native_inventory_budget() {
    let directory = temp_directory("ghostr-gateway-budget");

    let result =
        ffi_start_server(directory.to_string_lossy().to_string(), 1, 0, String::new()).await;

    assert!(result.is_err());
    assert!(!directory.exists());
}
