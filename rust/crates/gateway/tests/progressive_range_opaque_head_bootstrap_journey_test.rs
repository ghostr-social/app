mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::{Method, StatusCode};
use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;
use tower::ServiceExt as _;

#[tokio::test]
async fn head_without_accept_ranges_is_resolved_by_an_admitted_206() {
    let origin = ProgressiveJourneyOrigin::with_range_opaque_head().await;
    let harness = ProgressiveDeliveryHarness::start("ghostr-progressive-range-opaque-head");
    harness.focus(vec![unknown_item("delivery-current", &origin.url)], 0);
    harness.wait_until_registered("delivery-current").await;
    let request = harness.request("delivery-current", "bytes=0-2047").await;

    let response = harness
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("valid test fixture");
    let status = response.status();
    let body = to_bytes(response.into_body(), 2_048)
        .await
        .expect("valid test fixture");

    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert!(!body.is_empty());
    let requests = origin.requests();
    assert!(requests
        .iter()
        .any(|request| request.method == Method::HEAD));
    assert!(requests.iter().any(|request| request.method == Method::GET));
}
