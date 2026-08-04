mod support;

use rust_lib_ghostr::api::engine_control::ffi_start_engine;
use support::fixtures::temp_directory;

#[tokio::test]
async fn rejects_an_unknown_data_usage_level_before_starting() {
    let directory = temp_directory("ghostr-engine-usage");

    let result = ffi_start_engine(
        directory.to_string_lossy().to_string(),
        String::new(),
        "turbo".to_owned(),
        1024,
    )
    .await;

    assert!(result.is_err());
    assert!(!directory.exists());
}
