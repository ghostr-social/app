mod support;

use support::{engine, fixtures::temp_directory};

#[tokio::test]
async fn rejects_a_second_native_gateway_start() {
    let directory = temp_directory("ghostr-reentry");
    let first = engine::start(&directory, 1024).await;
    let second = engine::start(&directory, 1024).await;

    assert!(first.is_ok());
    assert!(second.is_err());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
