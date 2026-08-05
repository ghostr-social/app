mod support;

use support::{engine, fixtures::temp_directory};

#[tokio::test]
async fn rejects_a_zero_native_inventory_budget() {
    let directory = temp_directory("ghostr-gateway-budget");

    let result = engine::start(&directory, 0).await;

    assert!(result.is_err());
    assert!(!directory.exists());
}
