mod support;

use rust_lib_ghostr::video::video::ffi_start_server;
use support::fixtures::temp_directory;

#[tokio::test]
async fn rejects_zero_native_download_parallelism() {
    let directory = temp_directory("ghostr-gateway-parallelism");

    let result = ffi_start_server(
        directory.to_string_lossy().to_string(),
        0,
        1024,
        String::new(),
    )
    .await;

    assert!(result.is_err());
    assert!(!directory.exists());
}
