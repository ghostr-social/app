mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::{Method, StatusCode};
use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test]
async fn rejected_head_cannot_block_an_admitted_bootstrap_get() {
    let origin = ProgressiveJourneyOrigin::with_rejected_head().await;
    let harness = ProgressiveDeliveryHarness::start("ghostr-progressive-head-rejected");
    harness.focus(vec![unknown_item("delivery-current", &origin.url)], 0);
    harness.wait_until_registered("delivery-current").await;
    let request = harness.request("delivery-current", "bytes=0-2047").await;

    let response = tokio::time::timeout(
        Duration::from_secs(2),
        harness.router.clone().oneshot(request),
    )
    .await
    .expect("gateway progress")
    .expect("gateway response");
    let status = response.status();
    let body = to_bytes(response.into_body(), 2_048).await.unwrap();
    let requests = origin.requests();
    let total = harness.delivery.store.total_len("delivery-current").await;
    let ranges = harness
        .delivery
        .store
        .present_ranges("delivery-current")
        .await;

    assert_eq!(
        status,
        StatusCode::PARTIAL_CONTENT,
        "requests={requests:?}, total={total:?}, ranges={ranges:?}"
    );
    assert!(!body.is_empty());
    assert!(requests
        .iter()
        .any(|request| request.method == Method::HEAD));
    assert!(requests.iter().any(|request| request.method == Method::GET));
}
