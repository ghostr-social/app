#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ghostr_delivery::delivery_events::DeliveryCommand;
use ghostr_partial_store::partial_range_store::OutOfSpace;
use tower::ServiceExt;

#[tokio::test]
async fn debug_storage_budget_can_force_and_release_capacity_pressure() {
    let mut harness = gateway_fixture::progressive::progressive_harness("debug-storage-pressure");

    update_budget(&harness, 16).await;
    assert_storage_wake(&mut harness);
    let error = harness
        .store
        .write_range("clip", 0, &[7; 17])
        .await
        .unwrap_err();
    assert!(error.downcast_ref::<OutOfSpace>().is_some());

    update_budget(&harness, 32).await;
    assert_storage_wake(&mut harness);
    harness
        .store
        .write_range("clip", 0, &[7; 17])
        .await
        .unwrap();
}

fn assert_storage_wake(harness: &mut gateway_fixture::progressive::ProgressiveHarness) {
    assert!(matches!(
        harness.debug_commands.try_control(),
        Some(DeliveryCommand::StorageChanged)
    ));
}

async fn update_budget(harness: &gateway_fixture::progressive::ProgressiveHarness, bytes: u64) {
    let body = format!(r#"{{"budget_bytes":{bytes}}}"#);
    let response = harness
        .router
        .clone()
        .oneshot(
            Request::put("/debug/api/storage")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
