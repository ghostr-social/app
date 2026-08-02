use rust_lib_ghostr::video::video::ffi_start_server;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn starts_an_offline_native_gateway_with_a_health_endpoint() {
    let directory = cache_directory();

    let result = ffi_start_server(
        directory.to_string_lossy().to_string(),
        1,
        1024,
        String::new(),
    )
    .await;

    let endpoint = result.expect("gateway endpoint");
    let response = reqwest::get(format!("http://{endpoint}/status"))
        .await
        .expect("health request");
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
    fs::remove_dir_all(directory).expect("remove cache");
}

fn cache_directory() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ghostr-gateway-{nonce}"))
}
