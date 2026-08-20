mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::StatusCode;
use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test]
async fn range_blind_body_reaches_the_real_gateway_before_the_origin_finishes() {
    let origin = ProgressiveJourneyOrigin::with_range_blind_split().await;
    let harness = ProgressiveDeliveryHarness::start("ghostr-range-blind-streaming");
    harness.focus(vec![unknown_item("range-blind", &origin.url)], 0);
    harness.wait_until_registered("range-blind").await;
    let request = harness.request("range-blind", "bytes=0-3").await;
    let response = tokio::spawn(harness.router.clone().oneshot(request));

    origin.wait_for_prefix().await;
    let response = tokio::time::timeout(Duration::from_millis(500), response)
        .await
        .expect("gateway headers before origin completion")
        .expect("gateway task")
        .expect("gateway response");
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    let body = tokio::time::timeout(
        Duration::from_millis(500),
        to_bytes(response.into_body(), 4),
    )
    .await
    .expect("gateway prefix before origin completion")
    .expect("gateway body");
    assert_eq!(body.as_ref(), origin.prefix());

    origin.release();
}
