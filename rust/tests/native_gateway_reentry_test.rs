use rust_lib_ghostr::video::video::ffi_start_server;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn rejects_a_second_native_gateway_start() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("ghostr-reentry-{nonce}"));
    let first = ffi_start_server(
        directory.to_string_lossy().to_string(),
        1,
        1024,
        String::new(),
    )
    .await;
    let second = ffi_start_server(
        directory.to_string_lossy().to_string(),
        1,
        1024,
        String::new(),
    )
    .await;

    assert!(first.is_ok());
    assert!(second.is_err());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
