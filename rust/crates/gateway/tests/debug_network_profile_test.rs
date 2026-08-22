#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::Body;
use axum::http::{header::CONTENT_TYPE, Method, Request};
use gateway_fixture::debug_response::json_request;
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::delivery_events::DeliveryCommand;
use serde_json::json;

#[tokio::test]
async fn debug_network_profile_can_be_changed_without_restart() {
    let mut harness = progressive_harness("ghostr-debug-network");
    let payload = json!({
        "bandwidth_kbps": 768,
        "latency_ms": 350,
        "packet_loss_bps": 2_500,
        "max_connections_per_host": 1
    });
    let request = Request::builder()
        .method(Method::PUT)
        .uri("/debug/api/network")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("request");

    let updated = json_request(&harness, request).await;

    assert_eq!(updated, payload);
    assert_eq!(harness.network.profile().bandwidth_kbps, 768);
    assert_eq!(harness.network.profile().packet_loss_bps, 2_500);
    assert!(matches!(
        harness.debug_commands.try_control(),
        Some(DeliveryCommand::NetworkChanged)
    ));
}
